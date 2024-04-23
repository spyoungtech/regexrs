use pyo3::prelude::*;
use pyo3::types::{PyNone, PyTuple};
use regex;

#[pyclass]
struct Match {
    #[pyo3(get)]
    string: String,
    #[pyo3(get)]
    re: Pattern,
    #[pyo3(get)]
    pos: usize,
    #[pyo3(get)]
    endpos: usize,
}

#[pymethods]
impl Match {
    #[pyo3(signature = (*args))]
    fn group(&self, py: Python<'_>, args: &PyTuple) -> PyResult<PyObject> {
        let caps = self
            .re
            .regex
            .captures(&self.string)
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("No match found"))?;

        if args.is_empty() {
            // No arguments, return the whole match (group 0)
            return Ok(caps
                .get(0)
                .map_or_else(|| py.None(), |m| m.as_str().into_py(py)));
        }

        let groups: Vec<PyObject> = args
            .iter()
            .map(|g| match g.extract::<usize>() {
                Ok(index) => caps
                    .get(index)
                    .map_or_else(|| py.None(), |m| m.as_str().into_py(py)),
                Err(_) => py.None(),
            })
            .collect();

        if groups.len() == 1 {
            Ok(groups[0].clone_ref(py))
        } else {
            let tuple = PyTuple::new(py, &groups);
            Ok(tuple.to_object(py))
        }
    }

    fn groups(&self, py: Python<'_>) -> PyResult<PyObject> {
        let caps = self
            .re
            .regex
            .captures(&self.string)
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("No match found"))?;

        let groups: Vec<PyObject> = caps
            .iter()
            .skip(1) // Skip the entire match which is at index 0
            .map(|m| match m {
                Some(matched) => matched.as_str().into_py(py),
                None => py.None(),
            })
            .collect();

        Ok(PyTuple::new(py, &groups).to_object(py))
    }
}

#[pyclass]
#[derive(Clone)]
struct Pattern {
    // #[pyo3(get)]
    // flags: i32,
    regex: regex::Regex,
}

impl Pattern {
    fn new(pattern: String) -> Self {
        Pattern {
            regex: regex::Regex::new(pattern.as_str()).unwrap(),
        }
    }
}



#[pymethods]
impl Pattern {
    pub fn r#match(&self, string: String, pos: Option<i32>) -> PyResult<Option<Match>> {
        // todo: implement with find_at when `pos` is provided
        let m = self.regex.find(string.as_str());
        match m {
            Some(matched) => {
                let r = Match {
                    string: String::from(matched.as_str()),
                    re: self.clone(),
                    pos: matched.start(),
                    endpos: matched.end(),
                };
                Ok(Some(r))
            }
            None => Ok(None),
        }
    }
}

fn python_regex_flags_to_inline(pattern: String, flags: i32) -> String {
    // Define the flags based on the Python re module flag values
    const IGNORECASE: i32 = 2; // re.I or re.IGNORECASE
    const MULTILINE: i32 = 8; // re.M or re.MULTILINE
    const DOTALL: i32 = 16; // re.S or re.DOTALL
    const VERBOSE: i32 = 64; // re.X or re.VERBOSE

    let mut result = String::new();

    // Start the inline flag string
    result.push_str("(?");

    if flags & IGNORECASE != 0 {
        result.push('i');
    }
    if flags & MULTILINE != 0 {
        result.push('m');
    }
    if flags & DOTALL != 0 {
        result.push('s');
    }
    if flags & VERBOSE != 0 {
        result.push('x');
    }

    // Close the inline flag string
    result.push(')');

    // Return the resulting inline flags or an empty string if no flags are set
    if result.len() > 2 {
        return format!("{}{}", result, pattern)
    } else {
        return pattern
    }
}


#[pyfunction]
fn compile(pattern: String, flags: Option<i32>) -> Pattern {
    match flags {
        Some(given_flags) => {
            Pattern::new(python_regex_flags_to_inline(pattern, given_flags))
        },
        None => {
            Pattern::new(pattern)
        }
    }
}


#[pymodule]
fn regexrs(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Pattern>()?;
    m.add_class::<Match>()?;
    m.add("NOFLAG", 0);
    m.add("IGNORECASE", 2);
    m.add("I", 2);
    m.add("MULTILINE", 8);
    m.add("M", 8);
    m.add("DOTALL", 16);
    m.add("S", 16);
    m.add("VERBOSE", 64);
    m.add("X", 64);
    m.add_function(wrap_pyfunction!(compile, m)?)?;
    Ok(())
}
