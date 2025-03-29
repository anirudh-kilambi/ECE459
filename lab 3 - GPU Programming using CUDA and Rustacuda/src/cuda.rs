// This is the skeleton for the CUDA implementation

use crate::cnn::*;
use rustacuda::function::{BlockSize, GridSize};
use rustacuda::launch;
use rustacuda::memory::DeviceBox;
use rustacuda::prelude::*;
use std::error::Error;
use std::ffi::CString;

// Fields need to be ordered this way so the DeviceBoxes are
// dropped before the Context. Otherwise the drop will panic.

pub struct CudaContext {
    conv_layer: DeviceBox<ConvLayer>,
    output_layer: DeviceBox<OutputLayer>,
    module: Module,
    stream: Stream,
    _context: Context,
}

impl CudaContext {
    pub fn init(cnn: &Cnn) -> Result<Self, Box<dyn Error>> {
        rustacuda::init(CudaFlags::empty())?; 
        // Get the device (from the RustaCUDA documentation)
        // 0 is the device ID, we've got one GPU. 
        let device = Device::get_device(0)?; 
        // Initialize a CUDA context.
        // CUDA context is analagous to a CPU thread. 
        let context = Context::create_and_push(
            ContextFlags::MAP_HOST | ContextFlags::SCHED_AUTO,
            device,
        )?;
        // Load the module containing the kernel function
        let kernel_module_data = CString::new(include_str!("../kernel/kernel.ptx"))?;
        let kernel_module = Module::load_from_string(&kernel_module_data)?;

        // initialize a stream
        let stream = Stream::new(StreamFlags::DEFAULT, None)?;
        Ok(Self {
            conv_layer: DeviceBox::new(&cnn.conv_layer)?,
            output_layer: DeviceBox::new(&cnn.output_layer)?,
            module: kernel_module,
            stream,
            _context: context,
        })

    }

    pub fn compute(&mut self, input: &InputMatrix) -> Result<OutputVec, Box<dyn Error>> {
        let mut input_device = DeviceBox::new(input)?;

        let mut conv_output= ConvOutput([[[0.0; CONV_OUT_DIM]; CONV_OUT_DIM]; CONV_LAYER_SIZE]);
        let mut conv_output_device = DeviceBox::new(&conv_output)?;
        let conv_kernel_block= BlockSize::xyz(
            1 as u32,
            1 as u32,
            CONV_OUT_DIM as u32
        );
        let conv_kernel_grid = GridSize::xy(
            CONV_LAYER_SIZE as u32,
            CONV_OUT_DIM as u32
        );

        let relu_kernel_block = BlockSize::xyz(
            1 as u32,
            1 as u32,
            CONV_OUT_DIM as u32
        );

        let relu_kernel_grid = GridSize::xy(
            CONV_LAYER_SIZE as u32,
            CONV_OUT_DIM as u32
        );

        // initialize output vector as empty array of 4000 elements
        let mut output = OutputVec([0.0; OUT_LAYER_SIZE]);
        let mut output_device = DeviceBox::new(&output)?;
        // don't need to define a kernel_block_size because it's going to be 1 -- the default
        // output of the kernel is 4000x1
        let output_kernel_block = BlockSize::x(1);
        let output_kernel_grid = GridSize::x(OUT_LAYER_SIZE as u32);


        unsafe {
            // grab the conv, relu, and output kernel functions from `kernel.ptx`
            let conv_kernel = self.module.get_function(&CString::new("conv_layer")?)?;
            let relu_kernel = self.module.get_function(&CString::new("relu")?)?;
            let output_kernel = self.module.get_function(&CString::new("output")?)?;
            let stream = &self.stream;

            launch!(
                    conv_kernel<<<conv_kernel_grid, conv_kernel_block, 0, stream>>>(
                        input_device.as_device_ptr(), // this is just the shareable memory of the
                                                      // input matrix
                        self.conv_layer.as_device_ptr(), // this is just the shareable memory of
                                                         // the conv_layer
                        conv_output_device.as_device_ptr() // need an output, shape 10x20x20. This
                                                           // is initialized as empty (just 0.0)
                                                           // and will be filled in the kernel
                        )
                )?;
            launch!(
                    relu_kernel<<<relu_kernel_grid, relu_kernel_block, 0, stream>>>(
                        // This function just takes the output of the conv layer and sets negative
                        // values to 0. As a result the input can just be modified and returned.
                        // Will be done in place. No new data structure required.
                        conv_output_device.as_device_ptr() 
                        )
                )?;
            launch!(
                output_kernel<<<output_kernel_grid, output_kernel_block, 0, stream>>>(
                    // conv_output was also used for the output in the ReLU layer
                    conv_output_device.as_device_ptr(), 
                    // need the weights
                    self.output_layer.as_device_ptr(),
                    // This is the final output, it will be updated
                    output_device.as_device_ptr()
                    )
                )?;
        }

        self.stream.synchronize()?;

        output_device.copy_to(&mut output)?;
        Ok(output)
            

    }
}

