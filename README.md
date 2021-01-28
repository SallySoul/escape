# escape

Escape is a configurable renderer for the buddhabrot fractal.

## Usage

## Configuration Documenation

## Implementation Details

* Metropolis-Hastings based sampling, based on Alexander Boswell's work.
* Additionally, conjugate orbits used to inform sampling
* Multi-threaded

## Profiling

Followed [this article](https://gendignoux.com/blog/2019/11/09/profiling-rust-docker-perf.html), but its quite short if you ignore the docker bits.

```
cargo clean
RUSTFLAGS='-C force-frame-pointers=y' cargo build --release

perf record -g target/release/escape -- -c configs/config_sample.json -w 1
perf report -g graph,0.5,caller
```

## Resources / Further Reading

### [The Buddhabrot](http://www.steckles.com/buddha/) by Alexander Boswell

This implementation includes what is essentially a port of Alexander's sampling algorithm.
His work as described here is novel and often referenced in other writing about the buddhabrot set.
The key takeaway is to make use of the Metropolis-Hastings algorithm to enable expedient rendering of zoomed regions

### [The Buddhabrot Technique](http://superliminal.com/fractals/bbrot/bbrot.html) by Melinda Green

This is a comprehensive resource, including history of the technique and compiliation of other implementations.

### [https://benedikt-bitterli.me/buddhabrot/](https://benedikt-bitterli.me/buddhabrot/) by Benedikt Bitterli

And iteresting read with lots of pretty pictures and animations.
Benedikt's implementation and work is both GPU and animation focused.
He relates his importance sampling work to Alexander Boswell's algorith.

### [budhabrot - 2002](https://iquilezles.org/www/articles/budhabrot/budhabrot.html) by Inigo Quilez

Inigo's article describe his personal attempts working on rendering the buddha brot.
