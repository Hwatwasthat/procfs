use std::{collections::HashMap, ops::RangeInclusive};

use crate::{expect, from_str, FromBufRead, ProcError, ProcResult};

pub struct TttyDriver {
    pub name: String,
    pub node_name: String,
    pub major_number: isize,
    pub minor_numbers: RangeInclusive<isize>,
    pub driver_type: String,
}

impl TttyDriver {
    fn parse_line(line: &str) -> crate::ProcResult<Self> {
        let mut split = line.split_whitespace();
        let name = expect!(split.next()).to_string();
        let node_name = expect!(split.next()).to_string();
        let major_number = from_str!(isize, expect!(split.next()));
        let bounds = expect!(split.next());
        let minor_numbers = {
            if let Some((lower, upper)) = bounds.split_once("-") {
                let lower = from_str!(isize, lower);
                let upper = from_str!(isize, upper);
                lower..=upper
            } else {
                let single = from_str!(isize, bounds);
                single..=single
            }
        };
        let driver_type = expect!(split.next()).to_string();
        Ok(TttyDriver {
            name,
            node_name,
            major_number,
            minor_numbers,
            driver_type,
        })
    }
}

pub struct TtyDrivers {
    pub drivers: HashMap<String, TttyDriver>,
}

impl FromBufRead for TtyDrivers {
    fn from_buf_read<R: std::io::BufRead>(r: R) -> crate::ProcResult<Self> {
        let mut drivers = HashMap::new();
        for line in r.lines() {
            let line = line.map_err(|e| crate::ProcError::Other(e.to_string()))?;
            let driver = TttyDriver::parse_line(&line)?;
            let name = driver.name.clone();
            drivers.insert(name, driver);
        }
        Ok(TtyDrivers { drivers })
    }
}

pub struct LineDiscipline {
    pub name: String,
    pub no: usize,
}

impl LineDiscipline {
    fn parse_line(line: &str) -> ProcResult<Self> {
        let mut line = line.split_whitespace();
        let name = expect!(line.next()).to_string();
        let no_string = expect!(line.next());
        let no = from_str!(usize, no_string);
        Ok(LineDiscipline { name, no })
    }
}

pub struct LineDisciplines {
    pub disciplines: Vec<LineDiscipline>,
}

impl FromBufRead for LineDisciplines {
    fn from_buf_read<R: std::io::BufRead>(r: R) -> crate::ProcResult<Self> {
        let mut disciplines = Vec::new();
        for line in r.lines() {
            let line = line.map_err(|e| ProcError::Other(e.to_string()))?;
            let discipline = LineDiscipline::parse_line(&line)?;
            disciplines.push(discipline);
        }
        Ok(LineDisciplines { disciplines })
    }
}

#[cfg(test)]
mod test {
    use crate::FromBufRead;

    use super::{LineDiscipline, LineDisciplines, TttyDriver, TtyDrivers};

    #[test]
    fn correct_line_tty() {
        let line = "/dev/tty             /dev/tty        5       0 system:/dev/tty";
        let driver = TttyDriver::parse_line(line).expect("Did not parse line correctly");
        assert_eq!("/dev/tty", &driver.name);
        assert_eq!("/dev/tty", &driver.node_name);
        assert_eq!(5isize, driver.major_number);
        assert_eq!(0..=0, driver.minor_numbers);
        assert_eq!("system:/dev/tty", &driver.driver_type);
    }

    #[test]
    fn correct_file_tty() {
        let file = "/dev/tty             /dev/tty        5       0 system:/dev/tty
/dev/console         /dev/console    5       1 system:console
/dev/ptmx            /dev/ptmx       5       2 system
/dev/vc/0            /dev/vc/0       4       0 system:vtmaster
ttyAMA               /dev/ttyAMA   204 64-77 serial
ttyprintk            /dev/ttyprintk   5       3 console
pty_slave            /dev/pts      136 0-1048575 pty:slave
pty_master           /dev/ptm      128 0-1048575 pty:master
unknown              /dev/tty        4 1-63 console
";
        let drivers = TtyDrivers::from_buf_read(file.as_bytes()).expect("Unable to parse driver file string");
        assert_eq!(drivers.drivers.len(), 9);
        // Test one of the more complicated ones
        let pty_slave_driver = drivers
            .drivers
            .get("pty_slave")
            .expect("There should be a matching driver");
        assert_eq!(&pty_slave_driver.node_name, "/dev/pts");
        assert_eq!(pty_slave_driver.major_number, 136);
        assert_eq!(pty_slave_driver.minor_numbers, 0..=1048575);
        assert_eq!(&pty_slave_driver.driver_type, "pty:slave");
    }

    #[test]
    fn correct_ldisc_line() {
        let line = "n_tty       0";
        let ldisc = LineDiscipline::parse_line(line).expect("Could not parse Line Discipline");
        assert_eq!(&ldisc.name, "n_tty");
        assert_eq!(ldisc.no, 0);
    }

    #[test]
    fn correct_ldiscs_file() {
        let file_string = "n_tty       0
n_null     27";
        let ldiscs =
            LineDisciplines::from_buf_read(file_string.as_bytes()).expect("Unable to parse line discipline file string");
        assert_eq!(ldiscs.disciplines.len(), 2);
        let n_tty = ldiscs
            .disciplines
            .get(0)
            .expect("Could not access first line discipline from vec");
        let n_null = ldiscs
            .disciplines
            .get(1)
            .expect("Could not access second line discipline from vec");
        assert_eq!(&n_tty.name, "n_tty");
        assert_eq!(n_tty.no, 0);
        assert_eq!(&n_null.name, "n_null");
        assert_eq!(n_null.no, 27);
    }
}
