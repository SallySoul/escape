![FourPanel](four_panel.png?raw=true "FourPanel")

# escape

[![CI](https://github.com/sallysoul/escape/workflows/CI/badge.svg)](https://github.com/sallysoul/escape/actions)

Escape is a configurable renderer for the [buddhabrot](https://en.wikipedia.org/wiki/Buddhabrot).

## Installing

Gotta build it from source.
This requires the rust nightly toolchain installed.
If you do not have that installed, the go-to way of aquiring it is [rustup](https://rustup.rs).


```
# Currently using nightly, 1.51
rustup default nightly

# clone and build
git clone https://github.com/SallySoul/escape.git
cd escape
cargo install --path .
```

## Usage

There are 2 - 3 steps to getting pictures out of escape.

```
$ escape --help

```

### Sampling

```
$ escape sample --help

```

### Merging

```
$ escape merge --help

```

### Drawing

```
$ escape draw --help

```

## Implementation Details

* Metropolis-Hastings based sampling, based on Alexander Boswell's work.
* Additionally, conjugate orbits used to inform sampling
* Multi-threaded sampling with tokio
* Double precision floating points used (shrug)

## Resources / Further Reading

### [The Buddhabrot](http://www.steckles.com/buddha/) by Alexander Boswell

Alexander Boswell's work as described here is novel and often referenced in other writing about the buddhabrot set.
The key takeaway is to make use of the Metropolis-Hastings algorithm to enable expedient rendering of zoomed regions

### [Rendering a Buddhabrot at 4K and Other Bad Ideas](https://benedikt-bitterli.me/buddhabrot/) by Benedikt Bitterli

An iteresting read with lots of pretty pictures and animations.
Benedikt's implementation and work is both GPU and animation focused.
He relates his importance sampling work to Alexander Boswell's algorithm.
In addition, he talks about various approaches to smoothing the result, which I would like to incorporate.

### [The Buddhabrot Technique](http://superliminal.com/fractals/bbrot/bbrot.html) by Melinda Green

This is a comprehensive resource, including history of the technique and compiliation of other implementations.

### [budhabrot - 2002](https://iquilezles.org/www/articles/budhabrot/budhabrot.html) by Inigo Quilez

Inigo's article describe his personal attempts working on rendering the buddha brot.
