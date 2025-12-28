function setupPasteButton(): void {
    const pasteButton = document.getElementById('save-paste-btn') as HTMLButtonElement | null;
    const urlInput = document.getElementById('save-url-input') as HTMLInputElement | null;
    if (!pasteButton || !urlInput) {
        return;
    }

    pasteButton.addEventListener('click', async () => {
        if (!navigator.clipboard || !navigator.clipboard.readText) {
            alert('Clipboard access is not available');
            return;
        }

        try {
            const text = await navigator.clipboard.readText();
            if (text.length === 0) {
                return;
            }
            urlInput.value = text;
            urlInput.focus();
            urlInput.select();
        } catch (e) {
            console.error('Failed to read clipboard', e);
            alert('Could not read from clipboard');
        }
    });
}

export function setupSaveForm(): void {
    setupPasteButton();
}
