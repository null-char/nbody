#version 450

layout(location = 0) in vec2 a_position;
layout(location = 1) in vec3 a_color;
layout(location = 2) in vec2 center;
layout(location = 3) in float radius;
layout(location = 0) out vec3 v_color;

void main() {
  v_color = a_color;
  vec2 i_position = (radius * a_position) + center;
  gl_Position = vec4(i_position, 0.0, 1.0);
}