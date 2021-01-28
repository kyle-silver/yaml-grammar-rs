# Yamlfmt Integration Tests

Inside this directory you will find a `res/` folder with many sub-folders. When you want to create tests with a particular `.yamlfmt` file, create a folder with a descriptive, [kebab-cased](https://en.wikipedia.org/wiki/Letter_case#Special_case_styles) name. The format file should be named `spec.yamlfmt`, and all implementations of that format should exist in the same folder. You can then use `utils::spec()` and `utils::input()` to retrieve the contents in a standardized way.

```txt
.
├── src/
└── tests/
    ├── res/
    │   ├── nested-objects/
    │   │   ├── spec.yamlfmt
    │   │   ├── single-layer.yaml
    │   │   ├── multi-layer.yaml
    │   │   └── missing-required-fields.yaml
    │   └── type-mismatch/
    │       ├── spec.yamlfmt
    │       └── impl-01.yaml
    ├── secenarios.rs
    └── utils.rs
```
