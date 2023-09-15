varying vec2 v_uv;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;
uniform mat4 u_projection_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_transform;
void main() {
  v_uv = (a_pos + 1.0) / 2.0;
  gl_Position = u_projection_matrix * u_view_matrix * u_transform * vec4(a_pos, 0.0, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_texture;
uniform ivec2 u_texture_size;
uniform vec4 u_color;
void main() {
  vec4 color = texture2D(u_texture, v_uv);
  for (int dx = -1; dx <= 1; dx++) {
    for (int dy = -1; dy <= 1; dy++) {
      if (color != texture2D(u_texture, v_uv + dFdx(v_uv) * float(dx) + dFdy(v_uv) * float(dy))) {
        gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
        return;
      }
    }
  }
  discard;
}
#endif
