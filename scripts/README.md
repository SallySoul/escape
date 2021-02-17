# Scripts

I find it useful to play with creating images in a more dynamic environment.
These scripts are meant to be modified and run, they are not really tools like escape is.
To run the stuff in here you should need both python(3) and imagemagick (the `convert` utility specifically) installed.

## Making Videos

I found ffmpeg to be the easiest way to stitch frames into a video.

```
ffmpeg -framerate 24 -i 'frame_%04d/frame_2.png' -c:v libx264 -pix_fmt yuv420p m_circle_1.mp4
```

## Making STLs

I've found [fogelman/hmm](https://github.com/fogleman/hmm) to be an excellent tool for this job.
To build it I did have to to download [glm source](https://github.com/g-truc/glm), which is a header only library.
Then I had to patch the hmm Makefile.
```
diff --git a/Makefile b/Makefile
index a5618fd..f0634fb 100644
--- a/Makefile
+++ b/Makefile
@@ -8,7 +8,7 @@ SRC_EXT = cpp
 # Path to the source directory, relative to the makefile
 SRC_PATH = src
 # General compiler flags
-COMPILE_FLAGS = -std=c++11 -flto -O3 -Wall -Wextra -Wno-sign-compare -march=native
+COMPILE_FLAGS = -std=c++11 -flto -O3 -Wall -Wextra -Wno-sign-compare -march=native -I/home/russell/software/glm
 # Additional release-specific flags
 RCOMPILE_FLAGS = -D NDEBUG
 # Additional debug-specific flags

```

I found this to be a good invocation:
```
hmm \
    montage.png \
    montage.stl \
    --zscale 15.0 \
    --base 0.5 \
    --blur 2 \
    --border-size 20 \
    --triangles 160000 \
    --invert
```

## Head Zoom

[HeadZoom](https://gfycat.com/rectangularbitterherculesbeetle-buddhabrot-generative-fractal-rust)
