use crate::io;

const COLOR_CODE: char = '&';

pub fn wrap_and_clean(message: &str, default_mode: char) -> Vec<String>
{
	let mut output = Vec::new();
	let mut mode = default_mode;
	let mut new_mode = None;
	let mut spaces = 0;
	let len = message.chars().count();
	let mut i = 0;
	let mut split_at = None;
	let mut new_len = 0;
	let mut string = String::new();
	while i < len
	{
		if let Some(c) = message.chars().nth(i)
		{
			match c
			{
				'\n' =>
				{
					spaces = 0;
					output.push(string);
					string = String::new();
					if new_mode.is_none()
					{
						new_mode = Some(mode);
					}
					mode = default_mode;
					new_len = 0;
				}
				' ' =>
				{
					split_at = Some((i, mode, new_mode, string.len()));
					spaces += 1;
				}
				_ =>
				{
					if c == COLOR_CODE
					{
						if let Some(c) = message.chars().nth(i+1)
						{
							if c.is_ascii_hexdigit()
							{
								new_mode = Some(c.to_lowercase().next().unwrap());
								i += 2;
								continue;
							}
						}
					}
					new_len += 1 + spaces + if new_mode.is_some_and(|c| c != mode) { 2 } else { 0 };
					if new_len > io::STRING_LEN
					{
						if let Some((ni, nmode, nnew_mode, nlen)) = split_at
						{
							i = ni+1;
							mode = nmode;
							new_mode = nnew_mode;
							string.truncate(nlen);
							split_at = None;
						}
						output.push(string);
						string = String::new();
						spaces = 0;
						if new_mode.is_none()
						{
							new_mode = Some(mode);
						}
						mode = default_mode;
						new_len = if new_mode.is_some_and(|c| c != mode) { 3 } else { 1 };
						continue;
					}
					else
					{
						if spaces > 0
						{
							for _ in 0..spaces
							{
								string.push(' ');
							}
							spaces = 0;
						}
					}
					if let Some(c) = new_mode
					{
						if c != mode
						{
							mode = c;
							string.push(COLOR_CODE);
							string.push(c);
						}
						new_mode = None;
					}
					string.push(c);
				}
			}
		}
		else
		{
			unreachable!();
		}	
		i += 1;
	}
	if !string.is_empty()
	{
		output.push(string);
	}
	output
}