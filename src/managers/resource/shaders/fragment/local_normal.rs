pub const SHADER: &str = r#"
    precision mediump float;
    
    varying lowp vec3 vNormal;
    varying lowp vec4 vColor;
    varying lowp vec2 vUV0;
    varying lowp vec2 vUV1;

    void main() {
        gl_FragColor = vec4(vNormal, 1.0);
    }
"#;
