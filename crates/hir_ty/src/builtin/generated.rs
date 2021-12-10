//! Generated by `generate_builtins`, do not edit by hand.

use hir_def::BuiltIn;

use crate::builtin::*;

const BUILTIN_INFO: [BuiltinInfo; 116usize] = [
    ABS,
    ACOS,
    ACOSH,
    ASIN,
    ASINH,
    ATAN,
    ATAN2,
    ATANH,
    COS,
    COSH,
    EXP,
    FLOOR,
    FLOW,
    POTENTIAL,
    HYPOT,
    LN,
    LOG,
    MAX,
    MIN,
    POW,
    SIN,
    SINH,
    SQRT,
    TAN,
    TANH,
    DISPLAY,
    STROBE,
    WRITE,
    MONITOR,
    DEBUG,
    FCLOSE,
    FOPEN,
    FDISPLAY,
    FWRITE,
    FSTROBE,
    FMONITOR,
    FGETS,
    FSCANF,
    SWRITE,
    SFORMAT,
    SSCANF,
    REWIND,
    FSEEK,
    FTELL,
    FFLUSH,
    FERROR,
    FEOF,
    FDEBUG,
    FINISH,
    STOP,
    FATAL,
    WARNING,
    ERROR,
    INFO,
    ABSTIME,
    DIST_CHI_SQUARE,
    DIST_EXPONENTIAL,
    DIST_POISSON,
    DIST_UNIFORM,
    DIST_ERLANG,
    DIST_NORMAL,
    DIST_T,
    RANDOM,
    ARANDOM,
    RDIST_CHI_SQUARE,
    RDIST_EXPONENTIAL,
    RDIST_POISSON,
    RDIST_UNIFORM,
    RDIST_ERLANG,
    RDIST_NORMAL,
    RDIST_T,
    CLOG2,
    LOG10,
    CEIL,
    TEMPERATURE,
    VT,
    SIMPARAM,
    SIMPARAM_STR,
    SIMPROBE,
    DISCONTINUITY,
    MFACTOR,
    XPOSITION,
    YPOSITION,
    ANGLE,
    HFLIP,
    VFLIP,
    PARAM_GIVEN,
    PORT_CONNECTED,
    ANALOG_NODE_ALIAS,
    ANALOG_PORT_ALIAS,
    TEST_PLUSARGS,
    VALUE_PLUSARGS,
    ANALYSIS,
    AC_STIM,
    NOISE_TABLE,
    NOISE_TABLE_LOG,
    WHITE_NOISE,
    FLICKER_NOISE,
    LIMIT,
    BOUND_STEP,
    ABSDELAY,
    DDT,
    IDT,
    IDTMOD,
    DDX,
    ZI_ND,
    ZI_NP,
    ZI_ZD,
    ZI_ZP,
    LAPLACE_ND,
    LAPLACE_NP,
    LAPLACE_ZD,
    LAPLACE_ZP,
    LIMEXP,
    LAST_CROSSING,
    SLEW,
];
pub(crate) fn bultin_info(builtin: BuiltIn) -> BuiltinInfo { BUILTIN_INFO[builtin as u8 as usize] }