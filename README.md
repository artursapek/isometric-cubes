<p align=center><a href="https://art.cx"><img src="preview.png" alt="Preview" width=600 /></a></p>

# Isometric cube home page

This is a little Rust WASM toy I wrote for fun recently. It renders a bunch of cubes
which spell my name. They can be dragged around and push each other.

The isometric "physics" was written from scratch; it's very simple and not a full physics engine by any means.

It's written using [`web_sys`](https://docs.rs/web-sys/latest/web_sys/), and [`geo`](https://docs.rs/geo/latest/geo/) for basic geometric operations.

Try it live here: [https://art.cx](https://art.cx)

## Setup

To run the app:

```
npm install
npm run serve
```

The webpack server should start running at http://localhost:8080
