# wroomer - a simple zoomer application

<p align="center" width="100%">
  <video src="https://github.com/user-attachments/assets/55295f96-606e-4ad3-8b87-23da38a882f6" width="90%" controls></video>
</p>

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

## For nix users

The flake exposes 2 packages: `wroomer` and `wroomer-wayland` (which is actually an alias to `wroomer.override {waylandSupport = true;}`).

There is also a binary cache available:

```nix
nix.settings = {
  substituters = ["https://chilipizdrick.cachix.org"];
  trusted-public-keys = ["chilipizdrick.cachix.org-1:xVL2Q4Rbpc6EpDJ8lNHg7BMRhPfT26jw7l+jk4taUI8="];
};
```

## TODOs

- [x] Use `glam` crate instead of homegrown `Vec2f32` type.
- [x] Rewrite unhinged shader code to be 2 shaders + 2 draw calls.
- [x] Remove dvd logo code or make it more clean.
- [x] Implement image rotation (also in fixed steps).
- [x] Add github actions for automatic releases.
- [ ] Implement rendering high pixel count images (via multiple quads and textures).
