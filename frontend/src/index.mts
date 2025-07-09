import { currEntry } from './entry.mts';
import * as entry_list from './entry_list.mts';
import * as video from './video.mts';

addEventListener('DOMContentLoaded', () => {
    // TODO: only if the layout is cc-grid
    entry_list.setupPreviewFromCcGrid();
});

// Set up video stuff
addEventListener('DOMContentLoaded', () => {
    const entry = currEntry();
    if (entry === null || !entry.is_video) {
        return;
    }
    video.setupVideoPlayer();
});

// Utility function for writing to clipboard
function writeToClipboard(text: string) {
    if (navigator.clipboard) {
        navigator.clipboard.writeText(text)
            .then(() => { console.log("copy done"); })
            .catch(() => { alert("error"); });
    } else {
        let textArea = document.createElement("textarea");
        textArea.value = text;
        textArea.style.position = "absolute";
        textArea.style.opacity = "0";
        
        document.body.appendChild(textArea);
        textArea.focus();
        textArea.select();

        document.execCommand('copy');
        textArea.remove();
    }
}
(window as any).writeToClipboard = writeToClipboard;
