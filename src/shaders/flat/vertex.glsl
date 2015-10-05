#version 330

in vec3 position;
in vec3 normal;
in float value;

out vec3 v_normal;
out vec3 v_position;
out float v_value;

uniform mat4 perspective;
uniform mat4 view;
uniform mat4 model;

void main() {
    v_value = value;

    // position of vertex in clip space
    gl_Position = perspective * view * model * vec4(position, 1.0);

    // normal of vertex in camera space
    v_normal = transpose(inverse(mat3(view * model))) * normal;

    // position of vertex in camera space
    v_position = (view * model * vec4(position, 1.0)).xyz;
}
