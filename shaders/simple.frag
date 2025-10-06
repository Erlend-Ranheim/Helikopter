#version 430 core

in vec4 vertexColor;
in vec3 vertexNormals;
out vec4 color;



void main()
{
    vec3 lightDirection = normalize(vec3(0.8, -0.5, 0.6));

    float diffuse = max(0.0,dot(vertexNormals, -lightDirection));

    color = vec4(vec3(diffuse), 1.0);
}