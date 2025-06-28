import { currEntry } from './entry.mts';

export function markViewed(progress: number, ratio: number): void {
    // TODO: implement new sessions API
    // TODO: work for more than video
    const entry = currEntry();
    if (entry === null || !entry.is_video) {
        return;
    }

    jQuery.ajax(
        '/api/video_history',
        {
            type: "POST",
            data: JSON.stringify({
                // TODO(fyhuang): escape?
                "path": entry.repo_path,
                "current_ts": Math.floor(progress),
                "current_ratio": ratio,
            }),
            contentType: "application/json",
        },
    );
}