[Home](../index.md) / Why was it created

<div id="why" class="section" markdown="1">

## Why was BASCAL created

BASCAL traces its origins to 1985 and a preprocessor I wrote for a commercial BASIC development environment. We maintained a shared set of library routines across an application suite developed by a distributed team of two (at the time). Changes to those routines had to be merged manually into each developer's copy and subsequently into the applications that used them.

The preprocessor was written to address those problems. It allowed applications to be divided into multiple source files which were assembled into a single BASIC program for compilation. Shared library components could be maintained separately rather than copied and edited within each application. It also provided directives such as `@include`, `@if`, `@case`, `@function` and `@procedure`, together with `{label}` as an alternative to working directly with BASIC line numbers.

This made it considerably easier to maintain a suite of BASIC applications and to work across multiple developers and locations. Source files and common components could be managed independently, while the preprocessor dealt with assembling them into the form expected by the BASIC compiler.

I always intended to develop the idea further. My experience was not the obstacle, and the tools were adequate: the only feasible compiler construction set for DOS was QParser+, and while it was very capable, the time needed to implement the full BASCAL language with it was prohibitive. Four decades later, modern development tools, including Claude and Codex, made it practical to revisit the idea and implement BASCAL as a compiler rather than a text preprocessor.

The objectives remain much the same as they were in 1985: **make BASIC easier to write, structure and maintain, while respecting the environment in which it runs.** The original preprocessor allowed applications to be divided into multiple source files and assembled into a single BASIC program, making shared libraries easier to maintain and reducing the effort required to merge changes across multi-developer and distributed teams. BASCAL carries those ideas forward with structured source code, reusable components and a modern development model, while still producing code appropriate to its target environment.

The [full origin story](https://johnjoeallen.github.io/bascal/origin/) provides more background on the original preprocessor and the development of BASCAL.

</div>

---

[← Previous: What is BASCAL](../index.md) · [Next: How has it evolved →](evolution.md)
