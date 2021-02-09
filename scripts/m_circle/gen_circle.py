#!/usr/bin/env python3
import json
import os
import math

# Should we run commands that are generated (to escape and convert)
run_commands = True
release = True
workers = 14;
print_commands = True
recolor = False
gen_gif = False

reverse_gif = False;
rotate_gif_degrees = 0
gif_intermediate = "zoom_rotated.gif"
gif_output = "jc_circle_2.gif"

# Zoom / gif parameters
sample_config = {}
sample_config["cutoffs"] = [
    20, 50, 90, 150, 250
]
sample_config["julia_set_param"] = [0.0, 0.0]

sampling_duration = 0
frame_count = 0
gif_frame_delay = 0 # hundreths of a second
view = {}
view["zoom"] = 0.34
view["center"] = [0, 0]
if release:
    view["width"] = 2200
    view["height"] = 2200
    sample_config["samples"] = 10000
    sample_config["warm_up_samples"] = 10
    sampling_duration = 540
    frame_count = 220
    gif_frame_delay = 5
else:
    view["width"] = 100
    view["height"] = 100
    sample_config["samples"] = 1000
    sample_config["warm_up_samples"] = 100
    sampling_duration = 5
    frame_count = 20
    gif_frame_delay = 12

sample_config["view"] = view

# Sampling Parameters
# File paths and such
draw_config_path = os.getcwd() + "/color.json"
output_dir = os.getcwd()

for frame_index in range(0, frame_count):
    angle = ((frame_count - frame_index) / frame_count) * 2 * math.pi
    m_r = math.cos(angle)
    m_i = math.sin(angle)
    message = "Frame: " + str(frame_index) \
        + " angle: " + str(angle)

    working_dir = output_dir + "/frame_" + str(frame_index)
    frame_histogram_path = working_dir + "/histogram.json"
    frame_path = working_dir + "/frame.png"

    if not recolor:
        # Make working dir
        os.mkdir(working_dir)

        # In working dir, create sample_config
        sample_config["mandelbrot_param"] = [m_r, m_i]
        json.dumps(sample_config)
        frame_sample_config_path = working_dir + "/sample.json"
        frame_sample_config_file = open(frame_sample_config_path, "x")
        frame_sample_config_file.write(json.dumps(sample_config))
        frame_sample_config_file.close()

        # Run sampling for desired duration
        sample_command = "escape sample" \
        + " -c " + frame_sample_config_path \
        + " -w " + str(workers) \
        + " -d " + str(sampling_duration) \
        + " -o " + frame_histogram_path \
        + " -v off"

        if print_commands:
            print(sample_command)
        if run_commands:
            os.system(sample_command)

    # Render samples
    draw_command = "escape draw" \
    + " -c " + draw_config_path \
    + " -h " + frame_histogram_path \
    + " -o " + frame_path

    if print_commands:
        print(draw_command)
    if run_commands:
        os.system(draw_command)

# Once all frames are done, make a gif using convert
gif_command = "convert" \
    + " -loop 0" \
    + " -delay " + str(gif_frame_delay) \
    + " -dispose previous"

for i in range(0, frame_index):
    frame_path =  os.getcwd() + "/frame_" + str(i) + "/frame.png"
    gif_command += " " + frame_path

if reverse_gif:
    for i in range(1, frame_index - 1):
        index = frame_index - ( i + 1 )
        frame_path =  os.getcwd() + "/frame_" + str(index) + "/frame.png"
        gif_command += " " + frame_path

gif_command += " " + gif_intermediate

if print_commands:
    print(gif_command)
if run_commands:
    os.system(gif_command)

# We may want to rotate the gif
rotate_command = "convert" \
    + " " + gif_intermediate \
    + " -distort SRT " + str(rotate_gif_degrees) \
    + " " + gif_output

if print_commands and gen_gif:
    print(rotate_command)
if run_commands:
    os.system(rotate_command)
