![FourPanel](four_panel.png?raw=true "FourPanel")

# escape

[![CI](https://github.com/sallysoul/escape/workflows/CI/badge.svg)](https://github.com/sallysoul/escape/actions)

Escape contains configurable tools for sampling and rendering the [buddhabrot](https://en.wikipedia.org/wiki/Buddhabrot).

Escape is still under construction, and is not feature complete or stable.

The repo also includes some additional [scripts](scripts) for creating animations, etc.

## Usage

```
$ escape --help

```

There are 2 - 3 steps to getting renders out of escape.
First, a histogram file must be created using the `sample` command, in conjunction with a `SampleConfig` file.
Multiple histogram files can be merged with the `merge` command.
A histogram file can than be rendered using either the `draw` or `stl` commands using their respective config files.

### Sampling

```
$ escape sample --help

```

Sampling involves finding orbits that intersect the configured view.
The sample command will run-indefinitley until either `ctlr-c` is pressed or the a specified duration is up.
If running on a cluster I would recoment one process per machine with a worker for every logical core.

For example, on an eight core machine one might run sampling for four hours using the following command.
(Though `ctrl-c` could be passed before the time is up).

```
$ escape sample \
  --config configs/sample_config/AB_View_2.json \
  --output results/AB_View_2_histogram.json \
  --workers 8 \
  --duration 14400
```

Sampling is highly configurable, using a `SampleConfig` saved as a json file.
Examples of these files can be found in [`configs/sample_configs`](configs/sample_configs).

Note that the sampling method used by escape produces "splotchy" noise, particularly for deep zooms and high iteration cutoffs. Consider the three following frames that are differentiated only by adding additional sampling time.

![Splotchy](splotchy.gif?raw=true "Splotchy")

### Merging

```
$ escape merge --help

```

Merging results can be useful if the sampling time needs to be segmented over multiple intervals in time or machines in space.
For example, the sampling run for the title image was run across 40 machines in a cluster, which produced 40 histogram files.
The merge tool can combine these results into one file suitable for rendering.
As with sampling, the number of workers should at most be the number of logical cores, though if the number of results is smaller thats the maximum needed.

```
$ escape merge \
  --output merged_result.json \
  --worker 8 \
  result_hostname*.json
```

### Drawing

```
$ escape draw --help

```

Drawing is highly configurable, using a `DrawConfig` saved as a json file.
Examples of these files can be found in [`configs/draw_configs`](configs/draw_configs).

### Export as STL

```
$ escape stl --help
```

STL relief maps can be generated in a way similiar to the draw command.

## Building from source 

`escape` must be built rust 1.50.0 or greater, since it makes use of the `clamp` feature.
That's still a beta release as of writing.
If you do not have rust installed, the go-to way of aquiring it is [rustup](https://rustup.rs).


```
# Currently using 1.50.0, which is beta
rustup default beta

# clone and build
git clone https://github.com/SallySoul/escape.git
cd escape
cargo install --path .
```

## Implementation Details

* Metropolis-Hastings sampling
  - Enables effective sampling for deep zooms.
  - Configurable to avoid sampling bias / noise.
  - Based on Alexander Boswell's work.
* Conjugate orbits used to inform sampling.
  - Minor optimzation, evaluating orbits for high iteration cutoffs is expensive.
* Multi-threaded sampling with tokio.
  - No known limits to scaling, can take advantage of high core-count processors.
* Double precision floating points used (shrug)
  - I think it's neat.
  - Some processors might not.

## Resources / Further Reading

### [The Buddhabrot](http://www.steckles.com/buddha/) by Alexander Boswell


Alexander Boswell's work as described here is novel and often referenced in other writing about the buddhabrot.
The key takeaway is to make use of the Metropolis-Hastings algorithm to enable expedient rendering of zoomed regions.
In addition, the source Alexander released was critical for the development of this project.
Notable places I drew from this work include:

* Port of the metropolis-hastings mutation logic..
* Port of the algorithm for finding an initial sample for a metropolis hastings run.
* The suggested views in the source have been included as sample configs, `configs/sample/AB_View_*.json`.

### [Rendering a Buddhabrot at 4K and Other Bad Ideas](https://benedikt-bitterli.me/buddhabrot/) by Benedikt Bitterli

An iteresting read with lots of pretty pictures and animations.
Benedikt's implementation and work is both GPU and animation focused.
He relates his importance sampling work to Alexander Boswell's algorithm.
In addition, he talks about various approaches to smoothing the result, which I would like to incorporate.

### [The Buddhabrot Technique](http://superliminal.com/fractals/bbrot/bbrot.html) by Melinda Green

This is a comprehensive resource, including history of the technique and compiliation of other implementations.

### [budhabrot - 2002](https://iquilezles.org/www/articles/budhabrot/budhabrot.html) by Inigo Quilez

Inigo's article describe his personal attempts working on rendering the buddha brot.
I have always found Inigo's articles worth the read.
