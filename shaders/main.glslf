#version 150 core

uniform sampler2D t_Video;

in vec4 v_Color;
in vec2 v_Uv;
out vec4 Target0;

void main() {
    vec3 aw = texture(t_Video, v_Uv, 0).rgb;
    Target0 = vec4(aw, 1.0) * v_Color;
}