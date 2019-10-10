// Tested:
//
// - ✔ basic sending and receiving
// - ✔ try to read after close
// - ✔ try to write after close
// - ✔ read remaining data after close
//
use
{
	futures_ringbuf :: { *                                                                      } ,
	futures         :: { AsyncRead, AsyncWrite, AsyncWriteExt, AsyncReadExt, executor::block_on } ,
	futures_test    :: { task::noop_waker                                                       } ,
	assert_matches  :: { assert_matches                                                         } ,
	std             :: { task::{ Poll, Context }                                                } ,
	ergo_pin        :: { ergo_pin                                                               } ,
};



#[ test ]
//
fn basic_usage() { block_on( async
{
	let (mut server, mut client) = Endpoint::pair( 10, 10 );

	let     data = vec![ 1,2,3 ];
	let mut read = [0u8;3];

	server.write( &data ).await.expect( "write" );

	let n = client.read( &mut read ).await.expect( "read" );
	assert_eq!( n   , 3                 );
	assert_eq!( read, vec![ 1,2,3 ][..] );
})}


#[ test ] #[ ergo_pin ]
//
fn close_write()
{
	let (server, _client) = Endpoint::pair( 10, 10 );

	let     waker  = noop_waker();
	let mut cx     = Context::from_waker( &waker );
	let mut pserv  = pin!( server );

	let res = pserv.as_mut().poll_close( &mut cx );

	assert_matches!( res, Poll::Ready( Ok(_) ) );

	let buf = vec![ 1,2,3 ];
	let res = pserv.poll_write( &mut cx, &buf );

	match res
	{
		Poll::Ready( Err(e) ) => assert_eq!( e.kind(), std::io::ErrorKind::NotConnected ) ,
		_                     => assert!( false, "poll_write should return error: {:?}", res ),
	}
}


#[ test ] #[ ergo_pin ]
//
fn close_read()
{
	// flexi_logger::Logger::with_str( "futures_ringbuf=trace" ).start().expect( "flexi_logger");

	let (server, client) = Endpoint::pair( 10, 10 );

	let mut pserv  = pin!( server );
	let     pcl    = pin!( client );

	let     waker  = noop_waker();
	let mut cx     = Context::from_waker( &waker );

	// let res = pserv.as_mut().poll_close( &mut cx );
	// assert_matches!( res, Poll::Ready( Ok(_) ) );

	block_on( pserv.as_mut().close() ).expect( "close server" );

	let mut buf = [0u8;10];
	let res = pcl.poll_read( &mut cx, &mut buf );

	assert_matches!( res, Poll::Ready( Ok(0) ));
}



#[ test ]
//
fn close_read_remaining() { block_on( async
{
	let (mut server, mut client) = Endpoint::pair( 10, 10 );

	let     data  = vec![ 1,2,3 ];
	let mut read  = [0u8;3];
	let mut read2 = [0u8;3];

	server.write( &data ).await.expect( "write" );
	server.close().await.expect( "close" );

	let n = client.read( &mut read ).await.expect( "read" );
	assert_eq!( n   , 3                 );
	assert_eq!( read, vec![ 1,2,3 ][..] );

	let n = client.read( &mut read2 ).await.expect( "read" );
	assert_eq!( n   , 0        );
	assert_eq!( read2, [0u8;3] );
})}