export function test() {
    console.log("Hello, world!");
}

export function setupPreviewFromCcGrid(): void {
    const cards = document.querySelectorAll<HTMLDivElement>('.cc-card');
    const floatPreviewBox = document.getElementById('float-preview-box') as HTMLDivElement;
    const floatPreviewImage = document.getElementById('float-preview-img') as HTMLImageElement;

    cards.forEach(card => {
        const cardPreview = card.querySelector<HTMLImageElement>('img.cc-card-preview');
        const cardPreviewSrc = cardPreview?.src;
        if (!cardPreviewSrc) {
            return;
        }

        card.addEventListener('mouseover', (e: MouseEvent) => {
            // Determine which side of the screen to place the preview
            const isMouseInLeftHalf = e.clientX < window.innerWidth / 2;
            const isMouseInTopHalf = e.clientY < window.innerHeight / 2;

            // Set position of preview box
            floatPreviewBox.style.left = "auto";
            if (isMouseInTopHalf) {
                floatPreviewBox.style.top = "auto";
                floatPreviewBox.style.bottom = "0px";
            } else {
                floatPreviewBox.style.top = "0px";
                floatPreviewBox.style.bottom = "auto";
            }

            // Set the preview image source
            floatPreviewImage.src = cardPreviewSrc;

            // Show the preview box
            floatPreviewBox.style.display = 'block';
        });

        card.addEventListener('mouseout', () => {
            // Hide the preview box
            floatPreviewBox.style.display = 'none';
        });
    });
}