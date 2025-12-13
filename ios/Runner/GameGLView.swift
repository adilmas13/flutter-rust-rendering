import GLKit
import Flutter

class GameGLView: GLKView {
    private var gameHandle: UnsafeMutableRawPointer?
    private var displayLink: CADisplayLink?
    private var isInitialized = false

    override init(frame: CGRect) {
        guard let context = EAGLContext(api: .openGLES3) ?? EAGLContext(api: .openGLES2) else {
            fatalError("Failed to create EAGLContext")
        }
        super.init(frame: frame, context: context)
        setup()
    }

    required init?(coder: NSCoder) {
        guard let context = EAGLContext(api: .openGLES3) ?? EAGLContext(api: .openGLES2) else {
            fatalError("Failed to create EAGLContext")
        }
        super.init(coder: coder)
        self.context = context
        setup()
    }

    private func setup() {
        drawableColorFormat = .RGBA8888
        drawableDepthFormat = .format24
        drawableStencilFormat = .format8
        isMultipleTouchEnabled = true

        // Enable user interaction for touch handling
        isUserInteractionEnabled = true
    }

    override func layoutSubviews() {
        super.layoutSubviews()

        // Initialize or resize when layout changes
        EAGLContext.setCurrent(context)

        let width = UInt32(bounds.width * contentScaleFactor)
        let height = UInt32(bounds.height * contentScaleFactor)

        if !isInitialized && width > 0 && height > 0 {
            gameHandle = game_init(width, height)
            isInitialized = true
            startRenderLoop()
        } else if let handle = gameHandle {
            game_resize(handle, width, height)
        }
    }

    private func startRenderLoop() {
        displayLink = CADisplayLink(target: self, selector: #selector(renderFrame))
        displayLink?.preferredFramesPerSecond = 60
        displayLink?.add(to: .main, forMode: .common)
    }

    @objc private func renderFrame() {
        guard let handle = gameHandle else { return }
        EAGLContext.setCurrent(context)
        game_update(handle)
        display()
    }

    override func draw(_ rect: CGRect) {
        guard let handle = gameHandle else { return }
        game_render(handle)
    }

    func setDirection(_ direction: Int32) {
        guard let handle = gameHandle else { return }
        game_set_direction(handle, direction)
    }

    func setMode(_ mode: Int32) {
        guard let handle = gameHandle else { return }
        game_set_mode(handle, mode)
    }

    func setFps(_ fps: Int32) {
        if fps <= 0 {
            displayLink?.preferredFramesPerSecond = 0  // Device max
        } else {
            displayLink?.preferredFramesPerSecond = Int(fps)
        }
    }

    // MARK: - Touch Handling

    override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) {
        guard let touch = touches.first, let handle = gameHandle else { return }
        let location = touch.location(in: self)
        let scale = contentScaleFactor
        game_touch(handle, Float(location.x * scale), Float(location.y * scale), 0) // Down
    }

    override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent?) {
        guard let touch = touches.first, let handle = gameHandle else { return }
        let location = touch.location(in: self)
        let scale = contentScaleFactor
        game_touch(handle, Float(location.x * scale), Float(location.y * scale), 2) // Move
    }

    override func touchesEnded(_ touches: Set<UITouch>, with event: UIEvent?) {
        guard let touch = touches.first, let handle = gameHandle else { return }
        let location = touch.location(in: self)
        let scale = contentScaleFactor
        game_touch(handle, Float(location.x * scale), Float(location.y * scale), 1) // Up
    }

    override func touchesCancelled(_ touches: Set<UITouch>, with event: UIEvent?) {
        guard let touch = touches.first, let handle = gameHandle else { return }
        let location = touch.location(in: self)
        let scale = contentScaleFactor
        game_touch(handle, Float(location.x * scale), Float(location.y * scale), 1) // Up
    }

    deinit {
        displayLink?.invalidate()
        displayLink = nil
        if let handle = gameHandle {
            game_destroy(handle)
            gameHandle = nil
        }
    }
}
