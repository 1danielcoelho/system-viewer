attribute vec3 aPosition;
attribute vec3 aNormal;
attribute vec4 aColor;
attribute vec2 aUV0;
attribute vec2 aUV1;

uniform mat4 uTransform;

varying lowp vec3 vNormal;
varying lowp vec4 vColor;
varying lowp vec2 vUV0;
varying lowp vec2 vUV1;

void main() {
  vNormal = aNormal;
  vColor = aColor;
  vUV0 = aUV0;
  vUV1 = aUV1;

  gl_Position = uTransform * vec4(aPosition, 1.0);
}
