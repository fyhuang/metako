import { markViewed } from './history.mts';

export function setupVideoPlayer(): void {
    const mainVideo = document.querySelector('#mainvideo') as HTMLVideoElement;

    // Update history
    mainVideo.addEventListener('timeupdate', (event) => {
        markViewed(mainVideo.currentTime, mainVideo.currentTime / mainVideo.duration);
    });

    // YouTube-style keyboard shortcuts
    document.addEventListener('keyup', (event) => {
        // Play/pause/seek
        if (event.key === 'k') {
            if (mainVideo.paused) {
                mainVideo.play();
            } else {
                mainVideo.pause();
            }
        } else if (event.key === 'j') {
            mainVideo.currentTime -= 10;
        } else if (event.key === 'l') {
            mainVideo.currentTime += 10;
        }

        // Fullscreen
        if (event.key === 'f') {
            if (document.fullscreenElement === null) {
                mainVideo.requestFullscreen();
            } else {
                document.exitFullscreen();
            }
        }
    });
}
