varying vec2 v_uv;

#ifdef VERTEX_SHADER
attribute vec3 a_pos;
attribute vec2 a_uv;
uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_transform;
void main() {
  v_uv = a_uv;
  gl_Position = u_projection_matrix * u_view_matrix * u_transform * vec4(a_pos, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_texture;
uniform ivec2 u_texture_size;
uniform vec4 u_color;

void main() {
  gl_FragColor = smoothTexture2D(v_uv, u_texture, u_texture_size) * u_color;
  if (gl_FragColor.a < 0.5) {
    discard;
  }
}
#endif
