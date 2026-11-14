# Historical development notes (superseded)

The early prototype used text substitutions such as replacing `malloc` with `Box::new`. That rewrite was not semantically or memory-safe and has been removed. Crown now leaves pointer and allocation code visible for manual review. See the README for current behavior and limitations.
