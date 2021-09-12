#!/usr/bin/env python3
import json
import os

repo_location = "/home/russell/projects/rust-experiments/escape"

#output_dir = os.getcwd()
output_dir = "/home/russell/projects/escape_configs/study"

views_file = "configs/alex_views.json"

samples = [100, 1000, 10000, 100000, 1000000]
switch_prob = [0.01, 0.1, 0.2, 0.4]
cutoffs = [100, 1000, 10000, 100000, 1000000, 10000000]
width = 500
height = 500
sampling_duration = 15
workers = 7

print_commands = True
run_commands = True

views_file_path = repo_location + "/" + views_file
print("Views File Path: " + views_file_path)
with open(views_file_path) as views_file_handle:
   views = json.load(views_file_handle)

count = len(views) * len(samples) * len(switch_prob) * len(cutoffs)
print("Study Size: " + str(count))

def test_name(sample_count, cutoff, view_id):
    result = "study_sample" + str(sample_count) + "_cutoff" + str(cutoff) + "_view" + str(view_id)
    return result

def run_test(sample_count, cutoff, view_id):
    sample_config = {}
    sample_config["view"] = views[view_id]
    sample_config["view"]["width"] = width
    sample_config["view"]["height"] = height
    sample_config["cutoffs"] = [cutoff]
    sample_config["samples"] = sample_count


    working_dir = output_dir + "/" + test_name(sample_count, cutoff, view_id)

    if os.path.exists(working_dir):
        print("Working dir exists: " + working_dir)
        print("SKIPPING")
        return

    os.mkdir(working_dir)
    sample_config_path = working_dir + "/sample.json"
    sample_config_file = open(sample_config_path, "x")
    sample_config_file.write(json.dumps(sample_config))

    histogram_path = working_dir + "/histogram.json"

    sample_command = "/home/russell/.cargo/bin/escape sample" \
    + " -c " + sample_config_path \
    + " -w " + str(workers) \
    + " -d " + str(sampling_duration) \
    + " -o " + histogram_path \
    + " -v off"

    if print_commands:
        print(sample_command)
    if run_commands:
        wait_status = os.system(sample_command)
        if wait_status != 0:
            print("There was an error running sample command")
            exit(wait_status)

for sample_count in samples:
    for cutoff in cutoffs:
        for view_id in range(0, len(views)):
            run_test(sample_count, cutoff, view_id)

