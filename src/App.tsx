import { useEffect, useRef } from "react";
import init, { Chip8Engine } from "chip8-engine";

export function App() {
  const canvasRef = useRef<HTMLCanvasElement>(null);  
  const engineRef = useRef<Chip8Engine | null>(null);
  const isLoopRunningRef = useRef<boolean>(false);

  const audioCtxRef = useRef<AudioContext | null>(null);
  const gainNodeRef = useRef<GainNode | null>(null);

  const keymap: Record<string, number> = {
    "1": 0,
    "2": 1,
    "3": 2,
    "4": 3,
    "q": 4,
    "w": 5,
    "e": 6,
    "r": 7,
    "a": 8,
    "s": 9,
    "d": 10,
    "f": 11,
    "z": 12,
    "x": 13,
    "c": 14,
    "v": 15,
  }

  const SCREEN_WIDTH = 64;
  const SCREEN_HEIGHT = 32;
  const CANVAS_MULTIPLIER = 10;

  const oldEngine = false;

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
        if (e.key === "Escape"){
          engineRef.current = Chip8Engine.new(oldEngine);
          return;
        }

        const value = keymap[e.key];
        if (value === undefined) return;
        engineRef.current?.press_key(value);
      }

      const handleKeyUp = (e: KeyboardEvent) => {
        if (keymap[e.key] === undefined){
          return;
        }

        const value = keymap[e.key];
        if (value === undefined) return;
        engineRef.current?.release_key(value);
      };

    const startEmulator = async () => {
      await init("/rust_test_bg.wasm");
      
      engineRef.current = Chip8Engine.new(oldEngine);

      const canvas = canvasRef.current;
      if (!canvas) return;
      
      canvas.width = SCREEN_WIDTH * CANVAS_MULTIPLIER;
      canvas.height = SCREEN_HEIGHT * CANVAS_MULTIPLIER;

      window.addEventListener('keydown', handleKeyDown);

      window.addEventListener('keyup', handleKeyUp);

      console.log("Finished loading emulator");
    };

    startEmulator();

    return () => {
        window.removeEventListener('keydown', handleKeyDown);
        window.removeEventListener('keyup', handleKeyUp);
    };
  }, []); 

  const initAudio = () => {
    if (!audioCtxRef.current) {
      const audioCtx = new (window.AudioContext || (window as any).webkitAudioContext)();
      const oscillator = audioCtx.createOscillator();
      const gainNode = audioCtx.createGain();

      oscillator.type = "square"; 
      oscillator.frequency.value = 440; 

      gainNode.gain.value = 0;

      oscillator.connect(gainNode);
      gainNode.connect(audioCtx.destination);
      oscillator.start();

      audioCtxRef.current = audioCtx;
      gainNodeRef.current = gainNode;
    } else if (audioCtxRef.current.state === "suspended") {
      audioCtxRef.current.resume(); 
    }
  };

  const renderLoop = () => {
    if (engineRef.current) {
      for(let i = 0; i < 15; i++){
        engineRef.current.tick();
      }
      
      const delay = engineRef.current.get_delay_timer();
      if (delay > 0) engineRef.current.set_delay_timer(delay - 1);

      
      const sound = engineRef.current.get_sound_timer();
      if (sound > 0) {
        engineRef.current.set_sound_timer(sound - 1);
        if (gainNodeRef.current) {
            gainNodeRef.current.gain.value = 0.1; 
        }
      } else {
        if (gainNodeRef.current) {
            gainNodeRef.current.gain.value = 0; 
        }
      }

      const canvas = canvasRef.current;
      if (engineRef.current.get_draw_flag()) {
        if (canvas){
          const ctx = canvas.getContext("2d");
          const engineDisplay = engineRef.current.get_display();
          
          if(ctx){
            ctx.fillStyle = "#000000";
            ctx.fillRect(0, 0, canvas.width, canvas.height);

            ctx.fillStyle = "#FFFFFF";

            for(let i = 0; i < engineDisplay.length; i++){
              if (engineDisplay[i] == 1) {
                const y = Math.floor(i / SCREEN_WIDTH) * CANVAS_MULTIPLIER;
                const x = (i % SCREEN_WIDTH) * CANVAS_MULTIPLIER;
                
                ctx.fillRect(x, y, CANVAS_MULTIPLIER, CANVAS_MULTIPLIER);
              }
            }
          }
          
          engineRef.current.reset_draw_flag();
        }
      }
    };

    requestAnimationFrame(renderLoop);
  }

  const handleFileUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    initAudio(); 

    const buffer = await file.arrayBuffer();
    const romBytes = new Uint8Array(buffer);

    if (engineRef.current) {
      engineRef.current = Chip8Engine.new(oldEngine);
      engineRef.current.load_bytes(romBytes);
      console.log("Finished loading ROM!");
      
      if (!isLoopRunningRef.current) {
        isLoopRunningRef.current = true;
        renderLoop();
      }
    }
  };

  

  return (
    <div style={{ display: "flex", alignItems: "center", justifyContent: "center", marginTop: "50px", flexDirection: "column" }}>
      <canvas ref={canvasRef} style={{ border: "2px solid black" }}></canvas>
      
      <input type="file" accept=".ch8" onChange={handleFileUpload} />
    </div>
  );
}

export default App;