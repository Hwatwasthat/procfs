use std::{collections::HashMap, ops::RangeInclusive};

use crate::{expect, from_str, FromBufRead};

pub struct TttyDriver {
    pub name: String,
    pub node_name: String,
    pub major_number: isize,
    pub minor_numbers: RangeInclusive<isize>,
    pub driver_type: String,
}

pub struct TtyDrivers {
    pub drivers: HashMap<String, TttyDriver>,
}

impl FromBufRead for TtyDrivers {
    fn from_buf_read<R: std::io::BufRead>(r: R) -> crate::ProcResult<Self> {
        let mut drivers = HashMap::new();
        for line in r.lines() {
            let line = line.map_err(|e| crate::ProcError::Other(e.to_string()))?;
            let driver = parse_line(&line)?;
            let name = driver.name.clone();
            drivers.insert(name, driver);
        }
        Ok(TtyDrivers { drivers })
    }
}

fn parse_line(line: &str) -> crate::ProcResult<TttyDriver> {
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
    Ok(TttyDriver { name, node_name, major_number, minor_numbers, driver_type })
}

#[cfg(test)]
mod test {
    use crate::FromBufRead;

    #[test]
    fn correct_line() {
        let line = "/dev/tty             /dev/tty        5       0 system:/dev/tty";
        let driver = super::parse_line(line).expect("Did not parse line correctly");
        assert_eq!("/dev/tty", &driver.name);
        assert_eq!("/dev/tty", &driver.node_name);
        assert_eq!(5isize, driver.major_number);
        assert_eq!(0..=0, driver.minor_numbers);
        assert_eq!("system:/dev/tty", &driver.driver_type);
    }

    #[test]
    fn correct_file() {
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
        let drivers = super::TtyDrivers::from_buf_read(file.as_bytes()).expect("Unable to parse driver file string");
        assert_eq!(drivers.drivers.len(), 9);
        // Test one of the more complicated ones
        let pty_slave_driver = drivers.drivers.get("pty_slave").expect("There should be a matching driver");
        assert_eq!(&pty_slave_driver.node_name, "/dev/pts");
        assert_eq!(pty_slave_driver.major_number, 136);
        assert_eq!(pty_slave_driver.minor_numbers, 0..=1048575);
        assert_eq!(&pty_slave_driver.driver_type, "pty:slave");
    }
}
