#![no_std]
#![no_main]

#[cfg(feature = "defmt")]
use defmt::{debug, Format};
use embedded_hal_async::spi::Operation;
use embedded_hal_async::spi::SpiDevice;
use tartan_bitfield::bitfield;
use tartan_bitfield::Bitfield;

// ADS1220 SPI Commands
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum SpiCommand {
    Reset = 0x06,
    Start = 0x08,
    WriteReg = 0x40,
    ReadReg = 0x20,
}

#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum RegisterAddr {
    Reg0 = 0x00,
    Reg1 = 0x01,
    Reg2 = 0x02,
    Reg3 = 0x03,
}

bitfield! {
    #[cfg_attr(feature = "defmt", derive(Format))]
    pub struct Config0Reg(u8) {
        [4..=7] pub mux: u8 as AdcInputMux,
        [1..=3] pub gain: u8 as PgaGain,
        [0] pub pga_bypass,
    }
}

bitfield! {
    #[cfg_attr(feature = "defmt", derive(Format))]
    pub struct Config1Reg(u8) {
        [5..=7] pub data_rate: u8 as DataRate,
        [3..=4] pub operating_mode: u8 as OperatingMode,
        [2] pub conversion_mode,
        [1] pub temperature_sensor_mode,
        [0] pub burn_out_current_source,
    }
}

bitfield! {
    #[cfg_attr(feature = "defmt", derive(Format))]
    pub struct Config2Reg(u8) {
        [6..=7] pub vref_selection: u8 as VrefSelect,
        [4..=5] pub fir_filter: u8 as FIRRejectionFilter,
        [3] pub low_side_switch,
        [0..=2] pub idac_current_setting: u8 as IDacSourceCurrent,
    }
}

bitfield! {
    #[cfg_attr(feature = "defmt", derive(Format))]
    pub struct Config3Reg(u8) {
        [5..=7] pub idac1_mux: u8 as IDacRouting,
        [2..=4] pub idac2_mux: u8 as IDacRouting,
        [1] pub drdy_mode,
        [0] pub reserved,
    }
}

#[repr(u8)]
#[derive(Debug, num_enum::FromPrimitive, num_enum::IntoPrimitive)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum IDacRouting {
    #[default]
    Disabled = 0,
    Ain0RefP1 = 1,
    Ain1 = 2,
    Ain2 = 3,
    Ain3RefN1 = 4,
    RepP0 = 5,
    RefN0 = 6,
    Reserved = 7,
}

#[repr(u8)]
#[derive(Debug, num_enum::FromPrimitive, num_enum::IntoPrimitive)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum IDacSourceCurrent {
    #[default]
    Off = 0,
    Source10uA = 1,
    Source50uA = 2,
    Source100uA = 3,
    Source250uA = 4,
    Source500uA = 5,
    Source1000uA = 6,
    Source1500uA = 7,
}

#[repr(u8)]
#[derive(Debug, num_enum::FromPrimitive, num_enum::IntoPrimitive)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum FIRRejectionFilter {
    #[default]
    None = 0,
    Reject50and60Hz = 1,
    Reject50Hz = 2,
    Reject60Hz = 3,
}

#[repr(u8)]
#[derive(Debug, num_enum::FromPrimitive, num_enum::IntoPrimitive)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum VrefSelect {
    #[default]
    Internal2p048 = 0,
    ExternalRef0 = 1,
    ExternalRef1 = 2,
    AnalogSupply = 3,
}

#[repr(u8)]
#[derive(Debug, num_enum::FromPrimitive, num_enum::IntoPrimitive)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum OperatingMode {
    #[default]
    Normal = 0,
    DutyCycle = 1,
    Turbo = 2,
}

#[repr(u8)]
#[derive(Debug, num_enum::FromPrimitive, num_enum::IntoPrimitive)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum DataRate {
    #[default]
    Dr20sps = 0,
    Dr45sps = 1,
    Dr90sps = 2,
    Dr175sps = 3,
    Dr330sps = 4,
    Dr600sps = 5,
    Dr1000sps = 6,
}

#[repr(u8)]
#[derive(Debug, num_enum::FromPrimitive, num_enum::IntoPrimitive)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum PgaGain {
    #[default]
    Factor1 = 0,
    Factor2 = 1,
    Factor4 = 2,
    Factor8 = 3,
    Factor16 = 4,
    Factor32 = 5,
    Factor64 = 6,
    Factor128 = 7,
}

#[repr(u8)]
#[derive(Debug, num_enum::FromPrimitive, num_enum::IntoPrimitive)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum AdcInputMux {
    #[default]
    Ain0Ain1 = 0,
    Ain0Ain2 = 1,
    Ain0Ain3 = 2,
    Ain1Ain2 = 3,
    Ain1Ain3 = 4,
    Ain2Ain3 = 5,
    Ain1Ain0 = 6,
    Ain3Ain2 = 7,
    Ain0AVss = 8,
    Ain1AVss = 9,
    Ain2AVss = 10,
    Ain3AVss = 11,
    Ain0SingleEnded = 12,
    Ain1SingleEnded = 13,
    Ain2SingleEnded = 14,
    Ain3SingleEnded = 15,
}

pub struct ADS1220<SPI: SpiDevice> {
    spi: SPI,
}

impl<SPI: SpiDevice> ADS1220<SPI> {
    pub fn new(spi: SPI) -> Self {
        ADS1220 { spi }
    }

    async fn _write_register(
        &mut self,
        address: RegisterAddr,
        value: u8,
    ) -> Result<(), SPI::Error> {
        let write_op = [SpiCommand::WriteReg as u8 | ((address as u8) << 2), value];
        // defmt::info!("{:?}", write_op);
        let mut operations = [Operation::DelayNs(50), Operation::Write(&write_op)];
        self.spi.transaction(&mut operations).await
    }

    async fn write_register_0(&mut self, config: Config0Reg) -> Result<(), SPI::Error> {
        self._write_register(RegisterAddr::Reg0, config.value())
            .await
    }

    async fn write_register_1(&mut self, config: Config1Reg) -> Result<(), SPI::Error> {
        self._write_register(RegisterAddr::Reg1, config.value())
            .await
    }
    async fn write_register_2(&mut self, config: Config2Reg) -> Result<(), SPI::Error> {
        self._write_register(RegisterAddr::Reg2, config.value())
            .await
    }
    async fn write_register_3(&mut self, config: Config3Reg) -> Result<(), SPI::Error> {
        self._write_register(RegisterAddr::Reg3, config.value())
            .await
    }

    async fn _read_register(&mut self, register: RegisterAddr) -> Result<u8, SPI::Error> {
        let mut result: [u8; 1] = [0x00];
        let read_addr_vec = [SpiCommand::ReadReg as u8 | ((register as u8) << 2)];

        let master_dummy = [0xFF];

        let mut operations = [
            Operation::DelayNs(50),
            Operation::Write(&read_addr_vec),
            Operation::Transfer(&mut result, &master_dummy),
        ];
        self.spi.transaction(&mut operations).await?;
        Ok(result[0])
    }

    async fn read_register_0(&mut self) -> Result<Config0Reg, SPI::Error> {
        Ok(Config0Reg(self._read_register(RegisterAddr::Reg0).await?))
    }
    async fn read_register_1(&mut self) -> Result<Config1Reg, SPI::Error> {
        Ok(Config1Reg(self._read_register(RegisterAddr::Reg1).await?))
    }
    async fn read_register_2(&mut self) -> Result<Config2Reg, SPI::Error> {
        Ok(Config2Reg(self._read_register(RegisterAddr::Reg2).await?))
    }
    async fn read_register_3(&mut self) -> Result<Config3Reg, SPI::Error> {
        Ok(Config3Reg(self._read_register(RegisterAddr::Reg3).await?))
    }

    pub async fn begin(&mut self) -> Result<(), SPI::Error> {
        self.reset().await?;
        self.spi
            .transaction(&mut [Operation::DelayNs(50000)])
            .await?;

        self._write_register(RegisterAddr::Reg0, 0x00) // Default settings: AINP=AIN0, AINN=AIN1, Gain 1, PGA enabled
            .await?;
        self._write_register(RegisterAddr::Reg1, 0x04).await?; // Default settings: DR=20 SPS, Mode=Normal, Conv mode=continuous, Temp Sensor disabled, Current Source off
        self._write_register(RegisterAddr::Reg2, 0x10).await?; // Default settings: Vref internal, 50/60Hz rejection, power open, IDAC off
        self._write_register(RegisterAddr::Reg3, 0x00).await?; //  Default settings: IDAC1 disabled, IDAC2 disabled, DRDY pin only

        Ok(())
    }

    #[cfg(feature = "defmt")]
    pub async fn print_register_values(&mut self) -> Result<(), SPI::Error> {
        debug!("Config_Reg : ");
        debug!("{:#?}", self.read_register_0().await?);
        debug!("{:#?}", self.read_register_1().await?);
        debug!("{:#?}", self.read_register_2().await?);
        debug!("{:#?}", self.read_register_3().await?);
        Ok(())
    }

    pub async fn spi_command(&mut self, command: SpiCommand) -> Result<(), SPI::Error> {
        let data = [command as u8];
        let mut operations = [Operation::DelayNs(50), Operation::Write(&data)];
        self.spi.transaction(&mut operations).await
    }

    pub async fn reset(&mut self) -> Result<(), SPI::Error> {
        self.spi_command(SpiCommand::Reset).await
    }

    pub async fn start_conv(&mut self) -> Result<(), SPI::Error> {
        self.spi_command(SpiCommand::Start).await
    }

    pub async fn select_mux_channels(&mut self, mux_config: AdcInputMux) -> Result<(), SPI::Error> {
        let mut reg = self.read_register_0().await?;
        reg.set_mux(mux_config);
        self.write_register_0(reg).await
    }

    pub async fn set_pga_gain(&mut self, pga_gain: PgaGain) -> Result<(), SPI::Error> {
        let mut reg = self.read_register_0().await?;
        reg.set_gain(pga_gain);
        self.write_register_0(reg).await
    }

    pub async fn set_pga_on(&mut self) -> Result<(), SPI::Error> {
        let mut reg = self.read_register_0().await?;
        reg.set_pga_bypass(false);
        self.write_register_0(reg).await
    }

    pub async fn set_pga_off(&mut self) -> Result<(), SPI::Error> {
        let mut reg = self.read_register_0().await?;
        reg.set_pga_bypass(true);
        self.write_register_0(reg).await
    }

    pub async fn set_data_rate(&mut self, data_rate: DataRate) -> Result<(), SPI::Error> {
        let mut reg = self.read_register_1().await?;
        reg.set_data_rate(data_rate);
        self.write_register_1(reg).await
    }

    pub async fn set_operation_mode(&mut self, mode: OperatingMode) -> Result<(), SPI::Error> {
        let mut reg = self.read_register_1().await?;
        reg.set_operating_mode(mode);
        self.write_register_1(reg).await
    }

    pub async fn set_conv_mode_single_shot(&mut self) -> Result<(), SPI::Error> {
        let mut reg = self.read_register_1().await?;
        reg.set_conversion_mode(false); // Per datasheet
        self.write_register_1(reg).await
    }

    pub async fn set_conv_mode_continuous(&mut self) -> Result<(), SPI::Error> {
        let mut reg = self.read_register_1().await?;
        reg.set_conversion_mode(true); // Per datasheet
        self.write_register_1(reg).await
    }

    pub async fn temp_sensor_mode_disable(&mut self) -> Result<(), SPI::Error> {
        let mut reg = self.read_register_1().await?;
        reg.set_temperature_sensor_mode(false); // Per datasheet
        self.write_register_1(reg).await
    }

    pub async fn temp_sensor_mode_enable(&mut self) -> Result<(), SPI::Error> {
        let mut reg = self.read_register_1().await?;
        reg.set_temperature_sensor_mode(true); // Per datasheet
        self.write_register_1(reg).await
    }

    pub async fn current_sources_off(&mut self) -> Result<(), SPI::Error> {
        let mut reg = self.read_register_1().await?;
        reg.set_burn_out_current_source(false); // Per datasheet
        self.write_register_1(reg).await
    }

    pub async fn current_sources_on(&mut self) -> Result<(), SPI::Error> {
        let mut reg = self.read_register_1().await?;
        reg.set_burn_out_current_source(true); // Per datasheet
        self.write_register_1(reg).await
    }

    pub async fn set_vref(&mut self, vref: VrefSelect) -> Result<(), SPI::Error> {
        let mut reg = self.read_register_2().await?;
        reg.set_vref_selection(vref); // Per datasheet
        self.write_register_2(reg).await
    }

    pub async fn set_fir_filter(&mut self, filter: FIRRejectionFilter) -> Result<(), SPI::Error> {
        let mut reg = self.read_register_2().await?;
        reg.set_fir_filter(filter); // Per datasheet
        self.write_register_2(reg).await
    }

    pub async fn low_side_switch_open(&mut self) -> Result<(), SPI::Error> {
        let mut reg = self.read_register_2().await?;
        reg.set_low_side_switch(false); // Per datasheet
        self.write_register_2(reg).await
    }

    pub async fn low_side_switch_closed(&mut self) -> Result<(), SPI::Error> {
        let mut reg = self.read_register_2().await?;
        reg.set_low_side_switch(true); // Per datasheet, this closes the switch when START/SYNC command is sent and opens when the POWERDOWN command is issued
        self.write_register_2(reg).await
    }

    pub async fn set_idac_current(
        &mut self,
        idac_current: IDacSourceCurrent,
    ) -> Result<(), SPI::Error> {
        let mut reg = self.read_register_2().await?;
        reg.set_idac_current_setting(idac_current);
        self.write_register_2(reg).await
    }

    pub async fn set_idac1_route(&mut self, idac1_routing: IDacRouting) -> Result<(), SPI::Error> {
        let mut reg = self.read_register_3().await?;
        reg.set_idac1_mux(idac1_routing);
        self.write_register_3(reg).await
    }

    pub async fn set_idac2_route(&mut self, idac2_routing: IDacRouting) -> Result<(), SPI::Error> {
        let mut reg = self.read_register_3().await?;
        reg.set_idac2_mux(idac2_routing);
        self.write_register_3(reg).await
    }

    pub async fn set_drdy_mode_default(&mut self) -> Result<(), SPI::Error> {
        let mut reg = self.read_register_3().await?;
        reg.set_drdy_mode(false);
        self.write_register_3(reg).await
    }

    pub async fn set_drdy_mode_dout(&mut self) -> Result<(), SPI::Error> {
        let mut reg = self.read_register_3().await?;
        reg.set_drdy_mode(true);
        self.write_register_3(reg).await
    }

    pub async fn get_config_reg(&mut self) -> Result<[u8; 4], SPI::Error> {
        Ok([
            self.read_register_0().await?.value(),
            self.read_register_1().await?.value(),
            self.read_register_2().await?.value(),
            self.read_register_3().await?.value(),
        ])
    }

    pub async fn read_data_samples(&mut self) -> Result<[u8; 3], SPI::Error> {
        let mut buf: [u8; 3] = [0x00; 3];
        let mut operations = [Operation::DelayNs(50), Operation::Read(&mut buf)];
        self.spi.transaction(&mut operations).await?;
        Ok(buf)
    }

    pub fn data_to_int(&mut self, data: [u8; 3]) -> i32 {
        // extend to 32 bits
        let raw = i32::from_be_bytes([data[0], data[1], data[2], 0x00]);
        // resolve back to 24 bits, sign extension is automatic
        raw >> 8
    }

    pub async fn read_single_shot(&mut self) -> Result<i32, SPI::Error> {
        self.start_conv().await?;
        let data = self.read_data_samples().await?;
        Ok(self.data_to_int(data))
    }

    pub async fn read_single_shot_from_channel(
        &mut self,
        input_mux: AdcInputMux,
    ) -> Result<i32, SPI::Error> {
        self.select_mux_channels(input_mux).await?;
        self.read_single_shot().await
    }

    /*#[cfg(feature = "defmt")]
    const BYTES_PER_SAMPLE: u8 = 4;
    pub fn status(&mut self) {
        // let current_time: Instant = Instant::now();
        let ready = self.data_ready();
        let pending = if ready { BYTES_PER_SAMPLE } else { 0 };
        debug!("ads1220 status: ready={}, pending_bytes={:#X}", ready, pending)
    }*/
}
