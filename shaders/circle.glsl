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
uniform vec4 u_color;
void main() {
  if (length(v_pos) > 1.0) {
    discard;
  }
  gl_FragColor = u_color;
}
#endif
