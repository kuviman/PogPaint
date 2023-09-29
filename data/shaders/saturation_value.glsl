varying vec2 v_uv;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
uniform mat3 u_transform;
void main() {
  v_uv = a_pos * 0.5 + 0.5;
  gl_Position.xyw = u_projection_matrix * u_view_matrix * u_transform * vec3(a_pos, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER

uniform float u_hue;

float hsla_helper(float h, float l, float n, float alpha) {
    float k = n + h * 12.0;
    k = k - floor(k / 12.0) * 12.0;
    return l - alpha * max(-1.0, min(min(k - 3.0, 9.0 - k), 1.0));
}

vec4 hsla(float h, float s, float l, float a) {
  float alpha = s * min(l, 1.0 - l);
  return vec4(
    hsla_helper(h, l, 0.0, alpha),
    hsla_helper(h, l, 8.0, alpha),
    hsla_helper(h, l, 4.0, alpha),
    a);
}

void main() {
  gl_FragColor = hsla(u_hue, v_uv.x, v_uv.y, 1.0);
}
#endif
