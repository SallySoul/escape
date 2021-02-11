#!/usr/bin/env python3

import os
import copy
import json
import dominate
from dominate.tags import *

print("Starting")

sample_config_path = "sample.json"
histogram_path="histogram.json"
output_dir = os.getcwd()
preview_size = "200x200"
doc_path = "report.html"

sample_config_file = open(sample_config_path)
sample_config = json.load(sample_config_file)
sample_config_file.close()

print("About to make color_config")
color_config = {}
color_config["colors"] = []
color_config["powers"] = []
for cutoff in sample_config["cutoffs"]:
    color_config["colors"].append([0, 0, 0])
    color_config["powers"].append(1.0)
color_config["background_color"] = [0.0, 0.0, 0.0]

powers = [0.5, 1.0, 1.5]

doc = dominate.document(title = "Frame Study")
table = table()
header_row = tr()
header_row.add(td("Cutoff"))
for power in powers:
    header_row.add(td("power: " + str(power)))
table.add(header_row)

# Add table to document
print("About to start table gen")
for c_index, cutoff in enumerate(sample_config["cutoffs"]):
    cutoff_row = tr()
    cutoff_row.add(td(str(cutoff)))

    cutoff_color_config = copy.deepcopy(color_config)
    cutoff_color_config["colors"][c_index] = [255, 255, 255]

    for power in powers:
        print("cutoff: " + str(cutoff) + " power: " + str(power))
        working_dir = output_dir + "/frame_" + str(cutoff) + "_" + str(power)
        os.mkdir(working_dir)

        # Generate color config
        cutoff_color_config["powers"][c_index] = power
        color_config_path = working_dir + "/color.json"
        color_config_file = open(color_config_path, "x")
        color_config_file.write(json.dumps(cutoff_color_config))
        color_config_file.close()

        # Generate frame
        frame_path = working_dir + "/frame.png"
        draw_command = "escape draw" \
        + " -c " + color_config_path \
        + " -h " + histogram_path \
        + " -o " + frame_path

        print(draw_command)
        os.system(draw_command)

        # Generate reduced frame
        preview_path = working_dir + "/preview.png"
        resize_command = "convert" \
            + " " + frame_path \
            + " -resize " + preview_size \
            + " " + preview_path

        print(resize_command)
        os.system(resize_command)

        # Add to row
        image_element = img();
        image_element["src"] = preview_path
        link_element = a(image_element)
        link_element["href"] = frame_path
        cutoff_row.add( td(link_element))

    table.add(cutoff_row)

doc.add(table)
output_path = output_dir + "/" + doc_path
output_file = open(output_path, "x")
output_file.write(doc.render())
output_file.close()
