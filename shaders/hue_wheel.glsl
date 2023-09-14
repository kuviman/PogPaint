#include <color_wheel>

#ifdef FRAGMENT_SHADER
vec4 wheel_color(float x) {
  return hsla(x, u_actual_color_hsla.y, u_actual_color_hsla.z, 1.0);
}
#endif
