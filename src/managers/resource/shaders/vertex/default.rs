pub const SHADER: &str = r#"
    attribute vec3 aPosition;
    attribute vec3 aNormal;
    attribute vec4 aColor;
    attribute vec2 aUV0;
    attribute vec2 aUV1;

    uniform mat4 uTransform;

    varying lowp vec4 vColor;

    void main() {
        vColor = aColor;
        gl_Position = uTransform * vec4(aPosition, 1.0);
    }
"#;
