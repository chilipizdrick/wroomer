# wroomer - a simple zoomer application ~for wayland~ (now works everywhere)

This application is obviously inspired by [boomer](https://github.com/tsoding/boomer) by [tsoding](https://github.com/tsoding) and [woomer](https://github.com/coffeeispower/woomer) by [Tiago Dinis](https://github.com/coffeeispower) (which actually works on wayland).

## Controls

- Hold <kbd>Ctrl</kbd> - Turn spotlight on
- Right mouse button, <kbd>Esc</kbd> or <kbd>Q</kbd> - Quit application
- Left mouse button - Drag to move image
- Scroll wheel - Zoom image in/out
- <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + Scroll wheel - Adjust spotlight radius
- <kbd>D</kbd> - Spawn a DVD logo (only works with `--dvd-logo` cli flag)

## Why?

Why did I even write my version then? Well, fractional scaling on hyprland caused woomer's actual rendered window to be quarter of screen size due to a bug in GLFW, I suppose. And this inspired me to try out GPU programming with wgpu and create my own variant!

If you find this repository useful or inspiring, good for you, I guess.

## Planned features

If I have time and motivation, I will implement smooth scrolling and zooming and ~make application cross-platform~ it should be cross-platform.

# TODOs

- [ ] Use `glam` crate instead of homegrown `Vec2f32` type.
- [ ] Rewrite unhinged shader code to be 2 shaders + 2 draw calls.
- [ ] Remove dvd logo code or make it more clean.
- [ ] Implement image rotation.
- [ ] Add github actions for automatic releases.
- [ ] Add test coverage for critical parts of code.
