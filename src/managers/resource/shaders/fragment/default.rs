pub const SHADER: &str = r#"
    precision mediump float;
    
    varying lowp vec4 vColor;

    void main() {
        gl_FragColor = vColor + vec4(1.0, 0.0, 0.0, 0.0);
    }
"#;
