# wroomer - a simple zoomer application

This application is obviously inspired by [boomer](https://github.com/tsoding/boomer) by [tsoding](https://github.com/tsoding) and [woomer](https://github.com/coffeeispower/woomer) by [Tiago Dinis](https://github.com/coffeeispower) (which actually works on wayland).

## Controls

- Hold <kbd>Ctrl</kbd> - Turn spotlight on
- Right mouse button or <kbd>Esc</kbd> - Quit application
- Left mouse button - Drag to move image
- Scroll wheel - Zoom image in/out
- <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + Scroll wheel - Adjust spotlight radius
- <kbd>Alt</kbd> + Scroll wheel - Rotate image continuously
- <kbd>E</kbd> - Rotate image 90 degrees clockwise
- <kbd>Q</kbd> - Rotate image 90 degrees counterclockwise
- <kbd>R</kbd> - Reset image position

## Why?

Why did I even write my version then? Well, fractional scaling on hyprland caused woomer's actual rendered window to be quarter of screen size due to a bug in GLFW, I suppose. And this inspired me to try out GPU programming with wgpu and create my own variant!

If you find this repository useful or inspiring, good for you, I guess.

## Known bugs
- Application can only open images at most 8192 pixels in width OR height (seems to a WebGPU limitation..?)

## TODOs

- [x] Use `glam` crate instead of homegrown `Vec2f32` type.
- [x] Rewrite unhinged shader code to be 2 shaders + 2 draw calls.
- [x] Remove dvd logo code or make it more clean.
- [x] Implement image rotation (also in fixed steps).
- [ ] Add github actions for automatic releases.
- [ ] Implement rendering high pixel count images (more than 8192 in any dimension).
