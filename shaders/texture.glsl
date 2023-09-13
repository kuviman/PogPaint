varying vec2 v_uv;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
uniform mat3 u_transform;
void main() {
  v_uv = (a_pos + 1.0) / 2.0;
  gl_Position.xyw = u_projection_matrix * u_view_matrix * u_transform * vec3(a_pos, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_texture;
uniform ivec2 u_texture_size;
void main() {
  gl_FragColor = smoothTexture2D(v_uv, u_texture, u_texture_size);
}
#endif
