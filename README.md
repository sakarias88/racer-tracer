# Racer Tracer

A simple ray tracer written in rust. A hobby project for now. Feedback
and contributions are still very welcome.

![sample](./assets/trace_sample.png)

# Building


## Using nix
`nix-shell default.nix`
After that you can do `cargo build`, `cargo run --release` etc.


## Not using nix
Currently uses rust version 1.65.0(may work on lower versions).

On Linux you may need to install these dependencies for the `minifb`
crate to work.

`sudo apt install libxkbcommon-dev libwayland-cursor0 libwayland-dev`


# Running
The application accepts the following arguments.
`--config` path to the config file.
`--scene` path to the scene file (only supports yml).
`--image-action` (png, show).
    Png saves the resulting image to `image_output_dir`.
    Show just stops the rendering once its done and waits
    for you to press `R` again to continue with the real-time render.


Just running it without any arguments will use the default config and
scene provided by this repository. Once the application starts it will
present a crude preview image of the scene.

![preview](./assets/preview.png)

## Configuration
The configuration file has two blocks that controls the preview
quality and the render quality.  You can set the number of samples,
max_depth etc through threre.

## Controls
`WASD` Moves the camera in a currently crude way.
`R` Starts rendering the image.

## Rendering Progress
As you start rendering the image it will replace preview image with a
more refined one with the settings from the render block in your
configuration file.

![preview](./assets/preview.png)


![in_progress](./assets/in_progress.png)


![in_progress2](./assets/in_progress2.png)


Once the image is done rendering it will go forward with the selected
image action.
