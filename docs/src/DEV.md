# Development process

I decided to document how I do the dev process, since it's by now become core part of the project
itself.

## Atomic commits (and other Git things)

I fanatically split things into smaller commits when it's easy to do so.

- when it's not easy, make it easy
- when stuck/in doubt, reset everything, start over
- no branches
- no external issue tracking

## Abstraction

Here I have a somewhat similar view: try to split as much as possible.

- don't `dyn` for abstraction, only for necessary polymorphism (specifically the `Fetch`es)
- having only 1 or 0 concretions is not good enough to avoid an abstraction
- aim for encoding common interfaces as same trait even if they represent different things
  - this notably happened to the way we map `Extra`s
- custom logic for concretions should mostly be implemented in abstractions and then composed
  - see how schema thing does so
