# Webui core concepts

- Renderers are structs representing an object from a frontend-specific point of view. Computations related to display (e.g. based on the video info, display one icon or another) should be done in a Renderer rather than in the template itself.
- Partials abstract out common bits that can be reused across templates.
