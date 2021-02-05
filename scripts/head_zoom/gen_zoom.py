#!/usr/bin/env python3
import json
import os
import math

# Should we run commands that are generated (to escape and convert)
run_commands = True

# Should we print commands that are generated?
print_commands = True

# Zoom / gif parameters
start_zoom = 0.2
zoom_end = 900.0
frame_count = 80
gif_frame_delay = 12 # hundreths of a second
reverse_gif = True;
rotate_gif_degrees = 90

# Sampling Parameters
sampling_duration = 230;
workers = 15;

# File paths and such
gif_intermediate = "zoom_rotated.gif"
gif_output = "zoom.gif"
draw_config_path = os.getcwd() + "/color.json"
sample_config_path = os.getcwd() + "/view.json"
output_dir = os.getcwd()

sample_config_file = open(sample_config_path)
sample_config = json.load(sample_config_file)

# Zooming and How it Works:
#
# To achieve a visually even zoom transition
# one could apply a consistent multiplier each frame.
# But, how can we decide that we want to evenly split a zoom from a to b over n frames?
#
# We use a compact (takes [0, 1] parameter) and continuous function that describes the
# zoom transition for from the start to the end.
#
# We calculate the parameter for a given frame index with p_f as the fraction the total
# frame count.
# p_f(f) = (frame_count - f) / frame_count
#
# A zoom with a constant speed would make a linear relationship between the parameter and
# the logarithm of the zoom.
# log(p) = exp(p * (log(b) - log(a)) + log(a) where zoom : [0, 1] -> [a, b].
#
# However we want an eased transition that has smooth acceleration too!
# So we add the additional fuction that uses a cosine to smoothly acclerate the zoom.
# eased(p) = cos(p * PI) * 0.5 + 0.5
#
# Putting it together we use zoom(frame_index) = log(eased(p_f(frame_index)))
min_log = math.log(start_zoom)
max_log = math.log(zoom_end)
log_diff = max_log - min_log
for frame_index in range(0, frame_count):
    angle = (frame_count - frame_index) / frame_count * math.pi
    transitioned = math.cos(angle) * 0.5 + 0.5
    zoom = math.exp(transitioned * log_diff + min_log)
    message = "Frame: " + str(frame_index) \
        + " angle: " + str(angle) \
        + " trans: " + str(transitioned) \
        + " zoom: " + str(zoom)

    # Make working dir
    working_dir = output_dir + "/frame_" + str(frame_index)
    os.mkdir(working_dir)

    # In working dir, create sample_config
    sample_config["view"]["zoom"] = zoom
    json.dumps(sample_config)
    frame_sample_config_path = working_dir + "/sample.json"
    frame_sample_config_file = open(frame_sample_config_path, "x")
    frame_sample_config_file.write(json.dumps(sample_config))
    frame_sample_config_file.close()
    frame_histogram_path = working_dir + "/histogram.json"
    frame_path = working_dir + "/frame.png"

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

if print_commands:
    print(rotate_command)
if run_commands:
    os.system(rotate_command)
