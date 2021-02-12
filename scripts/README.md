# Scripts

I find it useful to play with creating images in a more dynamic environment.
These scripts are meant to be modified and run, they are not really tools like escape is.
To run the stuff in here you should need both python(3) and imagemagick (the `convert` utility specifically) installed.

## Making Videos

```
ffmpeg -framerate 24 -i 'frame_%04d/frame_2.png' -c:v libx264 -pix_fmt yuv420p m_circle_1.mp4
```

## Head Zoom

[HeadZoom](https://gfycat.com/rectangularbitterherculesbeetle-buddhabrot-generative-fractal-rust)
