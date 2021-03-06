#define KEEP_OPENGL_FUNCTIONS
#include "Thallium/Renderer/OpenGL/Windows/WindowsOpenGLRenderer.hpp"

#include <iostream>
#include <format>

#if defined(_WIN32) || defined(_WIN64)

namespace Thallium {

    Ref<Renderer> Renderer::CreateOpenGLRenderer(Ref<Window> window) {
        return Ref<WindowsOpenGLRenderer>::Create(window.As<WindowsWindow>());
    }

    WindowsOpenGLRenderer::WindowsOpenGLRenderer(Ref<WindowsWindow> window) : Window(window) {
        PIXELFORMATDESCRIPTOR pixelFormatDescriptor = {
            .nSize        = sizeof(pixelFormatDescriptor),
            .nVersion     = 1,
            .dwFlags      = PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
            .iPixelType   = PFD_TYPE_RGBA,
            .cColorBits   = 32,
            .cDepthBits   = 24,
            .cStencilBits = 8,
            .iLayerType   = PFD_MAIN_PLANE,
        };

        int format = ChoosePixelFormat(Window->DeviceContext, &pixelFormatDescriptor);
        if (format == 0) {
            std::cerr << std::format("Unable to choose pixel format: 0x{:x}", GetLastError()) << std::endl;
            std::exit(1);
        }

        if (!SetPixelFormat(Window->DeviceContext, format, &pixelFormatDescriptor)) {
            std::cerr << std::format("Unable to set pixel format: 0x{:x}", GetLastError()) << std::endl;
            std::exit(1);
        }

        OpenGLLibrary = LoadLibraryA("OpenGL32.dll");
        if (!OpenGLLibrary) {
            std::cerr << std::format("Unable load OpenGL32.dll: 0x{:x}", GetLastError()) << std::endl;
            std::exit(1);
        }

        HGLRC tempContext = wglCreateContext(Window->DeviceContext);
        if (!tempContext) {
            std::cerr << std::format("Unable create temp OpenGL context: 0x{:x}", GetLastError()) << std::endl;
            std::exit(1);
        }
        defer(wglDeleteContext(tempContext));

        if (!wglMakeCurrent(Window->DeviceContext, tempContext)) {
            std::cerr << std::format("Failed make temp OpenGL context current: 0x{:x}", GetLastError()) << std::endl;
            std::exit(1);
        }

        HGLRC(WINAPI * wglCreateContextAttribsARB)
        (HDC hDC, HGLRC hshareContext, const int* attribList) =
            reinterpret_cast<HGLRC (*)(HDC, HGLRC, const int*)>(wglGetProcAddress("wglCreateContextAttribsARB"));

        int attribs[] = {
            0,
        };

        OpenGLContext = wglCreateContextAttribsARB(Window->DeviceContext, nullptr, attribs);
        if (!OpenGLContext) {
            std::cerr << std::format("Unable create OpenGL context: 0x{:x}", GetLastError()) << std::endl;
            std::exit(1);
        }

        MakeContextCurrent();

    #define OPENGL_FUNCTION(ret, name, ...)                                                           \
        name##Func = reinterpret_cast<name##FunctionType*>(wglGetProcAddress(#name));                 \
        if (!name##Func) {                                                                            \
            name##Func = reinterpret_cast<name##FunctionType*>(GetProcAddress(OpenGLLibrary, #name)); \
            if (!name##Func) {                                                                        \
                std::cerr << std::format("Unable to get opengl function: '{}'", #name) << std::endl;  \
                std::exit(1);                                                                         \
            }                                                                                         \
        }
        OPENGL_FUNCTIONS
    #undef OPENGL_FUNCTION

        glEnable(GL_DEBUG_OUTPUT);
        glEnable(GL_DEBUG_OUTPUT_SYNCHRONOUS);
        glDebugMessageCallback(
            [](uint32_t source,
               uint32_t type,
               uint32_t id,
               uint32_t severity,
               uint32_t length,
               const char* message,
               void* userParam) {
                switch (severity) {
                    case GL_DEBUG_SEVERITY_HIGH: {
                        std::cerr << std::format("gl.DEBUG_SEVERITY_HIGH: '{}'\n", message) << std::endl;
                    } break;

                    case GL_DEBUG_SEVERITY_MEDIUM: {
                        std::cerr << std::format("gl.DEBUG_SEVERITY_MEDIUM: '{}'\n", message) << std::endl;
                    } break;

                    case GL_DEBUG_SEVERITY_LOW: {
                        std::cerr << std::format("gl.DEBUG_SEVERITY_LOW: '{}'\n", message) << std::endl;
                    } break;

                    case GL_DEBUG_SEVERITY_NOTIFICATION: {
                        std::cerr << std::format("gl.DEBUG_SEVERITY_NOTIFICATION: '{}'\n", message) << std::endl;
                    } break;

                    default: {
                        std::cerr << std::format("gl.DEBUG_SEVERITY_UNKNOWN: '{}'\n", message) << std::endl;
                    } break;
                }
            },
            nullptr);
    }

    WindowsOpenGLRenderer::~WindowsOpenGLRenderer() {
        wglDeleteContext(OpenGLContext);
        FreeLibrary(OpenGLLibrary);
    }

    void WindowsOpenGLRenderer::Present() {
        SwapBuffers(Window->DeviceContext);
    }

    void WindowsOpenGLRenderer::MakeContextCurrent() {
        if (wglGetCurrentContext() != OpenGLContext) {
            if (!wglMakeCurrent(Window->DeviceContext, OpenGLContext)) {
                std::cerr << std::format("Failed make OpenGL context current: 0x{:x}", GetLastError()) << std::endl;
                std::exit(1);
            }
        }
    }

}

#endif
