#version 430 core

layout (location = 0) in vec3 position;
layout (location = 1) in vec4 color;
layout (location = 2) in vec3 normals;

out vec4 vertexColor;
out vec3 vertexNormals;

uniform mat4 modelViewMatrix;
uniform mat4 modelMatrix;


void main()
{


    gl_Position = modelViewMatrix*vec4(position, 1.0f);

    mat3 modelMatrix3x3 = mat3(modelMatrix);
    vertexNormals = normalize(modelMatrix3x3 * normals);

    vertexColor = color;
}