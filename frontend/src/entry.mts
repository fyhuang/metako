// This is just a copy of EntryRenderer that can be accessed from JS
interface RenderedEntry {
    repo_path: string;

    is_video: boolean;
}

export function currEntry(): RenderedEntry | null {
    if (!('mtkEntry' in window)) {
        return null;
    }

    const mtkEntry = window.mtkEntry as RenderedEntry;
    return mtkEntry;
}