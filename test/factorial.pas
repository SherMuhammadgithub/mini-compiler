program factorial ( input , output ) ;
var n : integer ;
function fact ( k : integer ) : integer ;
begin
  if k = 0 then fact := 1
  else fact := k * fact ( k - 1 )
end ;
begin
  read ( n ) ;
  write ( fact ( n ) )
end .
