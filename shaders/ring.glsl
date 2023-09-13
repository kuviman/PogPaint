varying vec2 v_pos;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;
uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_transform;
void main() {
  v_pos = a_pos;
  gl_Position = u_projection_matrix * u_view_matrix * u_transform * vec4(a_pos, 0.0, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform vec4 u_color;
uniform float u_outer_radius;
uniform float u_inner_radius;
void main() {
  if (length(v_pos) > u_outer_radius) {
    discard;
  }
  if (length(v_pos) < u_inner_radius) {
    discard;
  }
  gl_FragColor = u_color;
}
#endif
