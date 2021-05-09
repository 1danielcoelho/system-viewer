# What is this?
This is a (very slightly) tweaked fork of https://crates.io/crates/egui_web to allow me to use egui as a tiny component of my web app, as opposed to being a wrapper around the app.

The two only changes are that this doesn't clear the target buffer before drawing, and it makes the WebBackend's `ctx` member public, so that I can pass it around my app to draw stuff.

It seems to share egui's MIT/Apache dual licenses so I copied them here.