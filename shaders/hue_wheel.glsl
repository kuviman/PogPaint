varying vec2 v_pos;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
uniform mat3 u_transform;
void main() {
  v_pos = a_pos;
  gl_Position.xyw = u_projection_matrix * u_view_matrix * u_transform * vec3(a_pos, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER

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

uniform float u_inner_radius;
uniform vec4 u_actual_color;
void main() {
  if (length(v_pos) > 1.0) {
    discard;
  }
  if (length(v_pos) < u_inner_radius) {
    gl_FragColor = u_actual_color;
    return;
  }
  float angle = atan(v_pos.y, v_pos.x);
  float hue = angle / (2.0 * PI);
  gl_FragColor = hsla(hue, 1.0, 0.5, 1.0);
}
#endif
