# YAML Grammar

This crate proposes an opinionated specification for turning the yaml-based config files for your project into a [Domain-Specific Language](https://en.wikipedia.org/wiki/Domain-specific_language) with built-in semantic validaitons. To do this, you first create a &ldquo;YAML Format&rdquo; file (a thing I made up) to specify the structure of your configuration file. You can then provide a copy of a configuration file along with your specification and evaluate it for soundness.

## Currently Supported

### Strings

* Allow and Disallow lists
* Equality / Inequality
* Regular Expressions

### Objects

* Grammar specification for sub-fields

## Under Development

### Numbers

* Allow and Disallow lists
* Equality / Inequality
* Comparisons
* Closed Ranges

### Booleans

* Equality / Inequality

### Sequences

* For-Each validations
* Length checking

## Unsupported

* Infinitely recursive types for Objects
* Specifying multiple discrete grammars for a single field
* Addressing for objects at specific indexes in a list
