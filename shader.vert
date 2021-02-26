#version 450

layout(location = 0) in vec2 a_position;
layout(location = 1) in vec3 a_color;
layout(location = 0) out vec3 v_color;

void main() {
  v_color = a_color;
  // mat4 ortho = mat4(
  //   2.0 / u_resolution.x, 0.0, 0.0, 0.0,
  //   0.0, 2.0 / u_resolution.y, 0.0, 0.0,
  //   0.0, 0.0, -2.0, 0.0,
  //   -1.0, -1.0, 1.0, 1
  // );
  gl_Position = vec4(a_position, 0.0, 1.0);
}