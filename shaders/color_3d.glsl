#ifdef VERTEX_SHADER
attribute vec2 a_pos;
uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_transform;
void main() {
  gl_Position = u_projection_matrix * u_view_matrix * u_transform * vec4(a_pos, 0.0, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform vec4 u_color;
void main() {
  gl_FragColor = u_color;
}
#endif
