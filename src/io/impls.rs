// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::{cmp, fmt, mem};

use crate::io::{self, Read, Seek, SeekFrom, Write};

// -----------------------------------------------------------------------------
// Forwarding implementations

impl<R: ?Sized + Read> Read for &mut R {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }
    // #[inline]
    // fn read_buf(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
    //     (**self).read_buf(cursor)
    // }
    // #[inline]
    // fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
    //     (**self).read_vectored(bufs)
    // }
    // #[inline]
    // fn is_read_vectored(&self) -> bool {
    //     (**self).is_read_vectored()
    // }
    // #[inline]
    // fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
    //     (**self).read_to_end(buf)
    // }
    // #[inline]
    // fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
    //     (**self).read_to_string(buf)
    // }
    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        (**self).read_exact(buf)
    }
    // #[inline]
    // fn read_buf_exact(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
    //     (**self).read_buf_exact(cursor)
    // }
}
impl<W: ?Sized + Write> Write for &mut W {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (**self).write(buf)
    }
    // #[inline]
    // fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
    //     (**self).write_vectored(bufs)
    // }
    // #[inline]
    // fn is_write_vectored(&self) -> bool {
    //     (**self).is_write_vectored()
    // }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        (**self).flush()
    }
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        (**self).write_all(buf)
    }
    // #[inline]
    // fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
    //     (**self).write_all_vectored(bufs)
    // }
    #[inline]
    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
        (**self).write_fmt(fmt)
    }
}
impl<S: ?Sized + Seek> Seek for &mut S {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        (**self).seek(pos)
    }
    #[inline]
    fn rewind(&mut self) -> io::Result<()> {
        (**self).rewind()
    }
    // #[inline]
    // fn stream_len(&mut self) -> io::Result<u64> {
    //     (**self).stream_len()
    // }
    // #[inline]
    // fn stream_position(&mut self) -> io::Result<u64> {
    //     (**self).stream_position()
    // }
    // #[inline]
    // fn seek_relative(&mut self, offset: i64) -> io::Result<()> {
    //     (**self).seek_relative(offset)
    // }
}

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<R: ?Sized + Read> Read for alloc::boxed::Box<R> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }
    // #[inline]
    // fn read_buf(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
    //     (**self).read_buf(cursor)
    // }
    // #[inline]
    // fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
    //     (**self).read_vectored(bufs)
    // }
    // #[inline]
    // fn is_read_vectored(&self) -> bool {
    //     (**self).is_read_vectored()
    // }
    // #[inline]
    // fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
    //     (**self).read_to_end(buf)
    // }
    // #[inline]
    // fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
    //     (**self).read_to_string(buf)
    // }
    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        (**self).read_exact(buf)
    }
    // #[inline]
    // fn read_buf_exact(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
    //     (**self).read_buf_exact(cursor)
    // }
}
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<W: ?Sized + Write> Write for alloc::boxed::Box<W> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (**self).write(buf)
    }
    // #[inline]
    // fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
    //     (**self).write_vectored(bufs)
    // }
    // #[inline]
    // fn is_write_vectored(&self) -> bool {
    //     (**self).is_write_vectored()
    // }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        (**self).flush()
    }
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        (**self).write_all(buf)
    }
    // #[inline]
    // fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
    //     (**self).write_all_vectored(bufs)
    // }
    #[inline]
    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> io::Result<()> {
        (**self).write_fmt(fmt)
    }
}
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl<S: ?Sized + Seek> Seek for alloc::boxed::Box<S> {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        (**self).seek(pos)
    }
    #[inline]
    fn rewind(&mut self) -> io::Result<()> {
        (**self).rewind()
    }
    // #[inline]
    // fn stream_len(&mut self) -> io::Result<u64> {
    //     (**self).stream_len()
    // }
    // #[inline]
    // fn stream_position(&mut self) -> io::Result<u64> {
    //     (**self).stream_position()
    // }
    // #[inline]
    // fn seek_relative(&mut self, offset: i64) -> io::Result<()> {
    //     (**self).seek_relative(offset)
    // }
}

// -----------------------------------------------------------------------------
// In-memory buffer implementations

/// Read is implemented for `&[u8]` by copying from the slice.
///
/// Note that reading updates the slice to point to the yet unread part.
/// The slice will be empty when EOF is reached.
impl Read for &[u8] {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let amt = cmp::min(buf.len(), self.len());
        let (a, b) = self.split_at(amt);

        // First check if the amount of bytes we want to read is small:
        // `copy_from_slice` will generally expand to a call to `memcpy`, and
        // for a single byte the overhead is significant.
        if amt == 1 {
            buf[0] = a[0];
        } else {
            buf[..amt].copy_from_slice(a);
        }

        *self = b;
        Ok(amt)
    }
    // #[inline]
    // fn read_buf(&mut self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
    //     let amt = cmp::min(cursor.capacity(), self.len());
    //     let (a, b) = self.split_at(amt);
    //
    //     cursor.append(a);
    //
    //     *self = b;
    //     Ok(())
    // }
    // #[inline]
    // fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
    //     let mut n_read = 0;
    //     for buf in bufs {
    //         n_read += self.read(buf)?;
    //         if self.is_empty() {
    //             break;
    //         }
    //     }
    //
    //     Ok(n_read)
    // }
    // #[inline]
    // fn is_read_vectored(&self) -> bool {
    //     true
    // }
    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        if buf.len() > self.len() {
            // `read_exact` makes no promise about the content of `buf` if it
            // fails so don't bother about that.
            *self = &self[self.len()..];
            return Err(io::Error::READ_EXACT_EOF);
        }
        let (a, b) = self.split_at(buf.len());

        // First check if the amount of bytes we want to read is small:
        // `copy_from_slice` will generally expand to a call to `memcpy`, and
        // for a single byte the overhead is significant.
        if buf.len() == 1 {
            buf[0] = a[0];
        } else {
            buf.copy_from_slice(a);
        }

        *self = b;
        Ok(())
    }
    // #[inline]
    // fn read_buf_exact(&mut self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
    //     if cursor.capacity() > self.len() {
    //         // Append everything we can to the cursor.
    //         cursor.append(*self);
    //         *self = &self[self.len()..];
    //         return Err(io::Error::READ_EXACT_EOF);
    //     }
    //     let (a, b) = self.split_at(cursor.capacity());
    //
    //     cursor.append(a);
    //
    //     *self = b;
    //     Ok(())
    // }
    // #[inline]
    // fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
    //     let len = self.len();
    //     buf.try_reserve(len)?;
    //     buf.extend_from_slice(*self);
    //     *self = &self[len..];
    //     Ok(len)
    // }
    // #[inline]
    // fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
    //     let content = str::from_utf8(self).map_err(|_| io::Error::INVALID_UTF8)?;
    //     let len = self.len();
    //     buf.try_reserve(len)?;
    //     buf.push_str(content);
    //     *self = &self[len..];
    //     Ok(len)
    // }
}

/// Write is implemented for `&mut [u8]` by copying into the slice, overwriting
/// its data.
///
/// Note that writing updates the slice to point to the yet unwritten part.
/// The slice will be empty when it has been completely overwritten.
///
/// If the number of bytes to be written exceeds the size of the slice, write operations will
/// return short writes: ultimately, `Ok(0)`; in this situation, `write_all` returns an error of
/// kind `ErrorKind::WriteZero`.
impl Write for &mut [u8] {
    #[inline]
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let amt = cmp::min(data.len(), self.len());
        let (a, b) = mem::take(self).split_at_mut(amt);
        a.copy_from_slice(&data[..amt]);
        *self = b;
        Ok(amt)
    }
    // #[inline]
    // fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
    //     let mut n_written = 0;
    //     for buf in bufs {
    //         n_written += self.write(buf)?;
    //         if self.is_empty() {
    //             break;
    //         }
    //     }
    //
    //     Ok(n_written)
    // }
    // #[inline]
    // fn is_write_vectored(&self) -> bool {
    //     true
    // }
    #[inline]
    fn write_all(&mut self, data: &[u8]) -> io::Result<()> {
        if self.write(data)? == data.len() { Ok(()) } else { Err(io::Error::WRITE_ALL_EOF) }
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Write is implemented for `Vec<u8>` by appending to the vector.
/// The vector will grow as needed.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl Write for alloc::vec::Vec<u8> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.extend_from_slice(buf);
        Ok(buf.len())
    }
    // #[inline]
    // fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
    //     let len = bufs.iter().map(|b| b.len()).sum();
    //     self.reserve(len);
    //     for buf in bufs {
    //         self.extend_from_slice(buf);
    //     }
    //     Ok(len)
    // }
    // #[inline]
    // fn is_write_vectored(&self) -> bool {
    //     true
    // }
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.extend_from_slice(buf);
        Ok(())
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Read is implemented for `VecDeque<u8>` by consuming bytes from the front of the `VecDeque`.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl Read for alloc::collections::VecDeque<u8> {
    /// Fill `buf` with the contents of the "front" slice as returned by
    /// [`as_slices`][`alloc::collections::VecDeque::as_slices`]. If the contained byte slices of the `VecDeque` are
    /// discontiguous, multiple calls to `read` will be needed to read the entire content.
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let (ref mut front, _) = self.as_slices();
        let n = Read::read(front, buf)?;
        self.drain(..n);
        Ok(n)
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let (front, back) = self.as_slices();

        // Use only the front buffer if it is big enough to fill `buf`, else use
        // the back buffer too.
        match buf.split_at_mut_checked(front.len()) {
            None => buf.copy_from_slice(&front[..buf.len()]),
            Some((buf_front, buf_back)) => match back.split_at_checked(buf_back.len()) {
                Some((back, _)) => {
                    buf_front.copy_from_slice(front);
                    buf_back.copy_from_slice(back);
                }
                None => {
                    self.clear();
                    return Err(io::Error::READ_EXACT_EOF);
                }
            },
        }

        self.drain(..buf.len());
        Ok(())
    }
    // #[inline]
    // fn read_buf(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
    //     let (ref mut front, _) = self.as_slices();
    //     let n = cmp::min(cursor.capacity(), front.len());
    //     Read::read_buf(front, cursor)?;
    //     self.drain(..n);
    //     Ok(())
    // }
    // fn read_buf_exact(&mut self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
    //     let len = cursor.capacity();
    //     let (front, back) = self.as_slices();
    //
    //     match front.split_at_checked(cursor.capacity()) {
    //         Some((front, _)) => cursor.append(front),
    //         None => {
    //             cursor.append(front);
    //             match back.split_at_checked(cursor.capacity()) {
    //                 Some((back, _)) => cursor.append(back),
    //                 None => {
    //                     cursor.append(back);
    //                     self.clear();
    //                     return Err(io::Error::READ_EXACT_EOF);
    //                 }
    //             }
    //         }
    //     }
    //
    //     self.drain(..len);
    //     Ok(())
    // }
    // #[inline]
    // fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
    //     // The total len is known upfront so we can reserve it in a single call.
    //     let len = self.len();
    //     buf.try_reserve(len)?;
    //
    //     let (front, back) = self.as_slices();
    //     buf.extend_from_slice(front);
    //     buf.extend_from_slice(back);
    //     self.clear();
    //     Ok(len)
    // }
    // #[inline]
    // fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
    //     // SAFETY: We only append to the buffer
    //     unsafe { io::append_to_string(buf, |buf| self.read_to_end(buf)) }
    // }
}

/// Write is implemented for `VecDeque<u8>` by appending to the `VecDeque`, growing it as needed.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
impl Write for alloc::collections::VecDeque<u8> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.extend(buf);
        Ok(buf.len())
    }
    // #[inline]
    // fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
    //     let len = bufs.iter().map(|b| b.len()).sum();
    //     self.reserve(len);
    //     for buf in bufs {
    //         self.extend(&**buf);
    //     }
    //     Ok(len)
    // }
    // #[inline]
    // fn is_write_vectored(&self) -> bool {
    //     true
    // }
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.extend(buf);
        Ok(())
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
