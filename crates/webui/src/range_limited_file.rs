use core::task::{Context, Poll};

use std::io::{Result, Error, ErrorKind};
use std::io::SeekFrom;
use std::pin::Pin;

use tokio::io::{AsyncSeek, AsyncRead};
use tokio::io::ReadBuf;

#[derive(PartialEq)]
enum SeekStatus {
    NeverSeeked,
    Seeking,
    SeekedOnce,
}

pub struct RangeLimitedFile<T> {
    pub source: T,
    pub start: u64,
    pub length: u64,

    seek_status: SeekStatus,
    read_position: u64,
}

impl <T> RangeLimitedFile<T> {
    pub fn new(source: T, start: u64, length: u64) -> RangeLimitedFile<T> {
        RangeLimitedFile {
            source,
            start,
            length,
            seek_status: SeekStatus::NeverSeeked,
            read_position: 0,
        }
    }
}

impl <T: AsyncSeek + AsyncRead + Send + std::marker::Unpin> AsyncSeek for RangeLimitedFile<T> {
    fn start_seek(mut self: Pin<&mut Self>, position: SeekFrom) -> Result<()> {
        let limited_pos = match position {
            SeekFrom::Start(pos) => SeekFrom::Start(self.start + pos),
            SeekFrom::End(pos) => {
                if pos < 0 && (-pos as u64) > self.length {
                    // Error to seek before start
                    return Err(Error::new(ErrorKind::InvalidInput, "Cannot seek before beginning"));
                }

                if pos < 0 {
                    SeekFrom::Start(self.start + self.length - (-pos as u64))
                } else {
                    SeekFrom::Start(self.start + self.length + (pos as u64))
                }
            }
            SeekFrom::Current(pos) => SeekFrom::Current(pos),
        };

        self.seek_status = SeekStatus::Seeking;
        Pin::new(&mut self.source).start_seek(limited_pos)
    }

    fn poll_complete(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<u64>> {
        match Pin::new(&mut self.source).poll_complete(cx) {
            Poll::Ready(result) => match result {
                Ok(new_pos) => {
                    self.read_position = new_pos;
                    println!("new_pos = {}", new_pos);
                    if new_pos < self.start {
                        return Poll::Ready(Ok(0));
                    }

                    if self.seek_status == SeekStatus::Seeking {
                        self.seek_status = SeekStatus::SeekedOnce;
                    }

                    let limited_pos = new_pos - self.start;
                    Poll::Ready(Ok(limited_pos))
                }
                Err(e) => Poll::Ready(Err(e))
            }
            Poll::Pending => Poll::Pending
        }
    }
}

impl <T: AsyncSeek + AsyncRead + Send + std::marker::Unpin> AsyncRead for RangeLimitedFile<T> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<Result<()>> {
        // Seek to start if we've never seeked before (file may start seeked to before start)
        if self.seek_status == SeekStatus::NeverSeeked {
            let seek_result = Pin::new(&mut self).start_seek(SeekFrom::Start(0));
            if seek_result.is_err() {
                return Poll::Ready(seek_result);
            }
        }

        if self.seek_status == SeekStatus::Seeking {
            match Pin::new(&mut self).poll_complete(cx) {
                Poll::Ready(seek_result) => {
                    if seek_result.is_err() {
                        return Poll::Ready(Err(seek_result.err().unwrap()));
                    }
                }
                Poll::Pending => { return Poll::Pending; }
            }
        }

        let end = self.start + self.length;
        let bytes_remaining = end - self.read_position;

        let pre_read_init = buf.initialized().len();

        let mut limited_buf = buf.take(bytes_remaining as usize);
        let source_result = Pin::new(&mut self.source).poll_read(cx, &mut limited_buf);
        let (new_init, new_filled) = match source_result {
            Poll::Ready(Err(e)) => { return Poll::Ready(Err(e)); }
            Poll::Pending => { return Poll::Pending; }
            Poll::Ready(Ok(())) => (limited_buf.initialized().len(), limited_buf.filled().len())
        };

        unsafe { buf.assume_init(pre_read_init + new_init) };
        buf.advance(new_filled);
        self.read_position += new_filled as u64;
        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use tokio::io::{AsyncSeekExt, AsyncReadExt};

    #[tokio::test]
    async fn test_full_limit() {
        let in_buf = [1u8, 2u8, 3u8, 4u8];
        let mut f = RangeLimitedFile::new(Cursor::new(in_buf), 0, 4);

        let mut out_buf = [0u8; 4];
        f.read(&mut out_buf).await.unwrap();

        assert_eq!(out_buf, in_buf);
    }

    #[tokio::test]
    async fn test_end_limited() {
        let in_buf = [1u8, 2u8, 3u8, 4u8];
        let mut f = RangeLimitedFile::new(Cursor::new(in_buf), 0, 2);

        let mut out_buf = [0u8; 4];
        f.read(&mut out_buf).await.unwrap();

        assert_eq!(out_buf, [1u8, 2u8, 0u8, 0u8]);
    }

    #[tokio::test]
    async fn test_both_limited() {
        let in_buf = [1u8, 2u8, 3u8, 4u8];
        let mut f = RangeLimitedFile::new(Cursor::new(in_buf), 1, 2);

        let mut out_buf = [0u8; 4];
        f.read(&mut out_buf).await.unwrap();

        assert_eq!(out_buf, [2u8, 3u8, 0u8, 0u8]);
    }

    #[tokio::test]
    async fn test_seek() {
        let in_buf = [1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8];
        let mut f = RangeLimitedFile::new(Cursor::new(in_buf), 1, 6);

        let mut out_buf = [0u8; 2];
        assert_eq!(f.seek(SeekFrom::Start(1)).await.unwrap(), 1);
        f.read(&mut out_buf).await.unwrap();
        assert_eq!(out_buf, [3u8, 4u8]);

        assert_eq!(f.seek(SeekFrom::Start(0)).await.unwrap(), 0);
        f.read(&mut out_buf).await.unwrap();
        assert_eq!(out_buf, [2u8, 3u8]);
        f.read(&mut out_buf).await.unwrap();
        assert_eq!(out_buf, [4u8, 5u8]);

        assert_eq!(f.seek(SeekFrom::End(-2)).await.unwrap(), 4);
        f.read(&mut out_buf).await.unwrap();
        assert_eq!(out_buf, [6u8, 7u8]);

        // Doesn't read past end
        out_buf = [0u8; 2];
        assert_eq!(f.seek(SeekFrom::End(-1)).await.unwrap(), 5);
        assert_eq!(f.read(&mut out_buf).await.unwrap(), 1);
        assert_eq!(out_buf, [7u8, 0u8]);
        assert_eq!(f.read(&mut out_buf).await.unwrap(), 0);

        // Cannot seek before beginning
        assert!(f.seek(SeekFrom::End(-7)).await.is_err());
    }
}
