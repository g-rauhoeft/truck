# truck: Truck is a rust CAD kernel

## Overview
Truck is an open-source shape processing kernel by Rust.
These crates are focused on performance, simplification and accessibility.

### performance
The whole package is written in Rust, with the algorithm as well as the timing of memory allocation and release explicitly chosen.

### simplification
I avoided abstraction and specialization as much as possible to make it easier to grasp the whole. For example, `track_geometry` does the following.
* The distinction between coordinates and vectors, as seen in the STEP standard, has been removed.
* We do not define memory optimization by CSG representation or Bezier splines, but use 4D B-spline to display all geometric information.

### accessibility
Truck is a collection of multiple crates. Users can use only the packages they need individually. For example:
* Even if an OpenGL viewer is incorporated in the future, web app developers will be able to create apps that are independent from that viewer.
* If you don't like the CSG-like utility API provided here, you can create your own utility that depends only on more primitive crates.

### crates
* [truck-geometry](https://ricos.pages.ritc.jp/truck/truck/truck-geometry/index.html)  
geometrical structs: vector, matrix, knot vector, and bsplines
* [truck-topology](https://ricos.pages.ritc.jp/truck/truck/truck-topology/index.html)  
topological structs: vertex, edge, wire, face, shell, and solid.
* [truck-polymesh](https://ricos.pages.ritc.jp/truck/truck/truck-polymesh/index.html)  
define polyline-polygon data structure and some algorithms handling mesh, including meshing the shapes
* [truck-io](https://ricos.pages.ritc.jp/truck/truck/truck-io/index.html)  
input / output libraries for obj, tts, tgb, and so on.
