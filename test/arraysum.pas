program arraysum ( input , output ) ;
var a : array [ 1 .. 10 ] of integer ;
var i , sum : integer ;
begin
  sum := 0 ;
  i := 1 ;
  while i <= 10 do
  begin
    read ( a [ i ] ) ;
    sum := sum + a [ i ] ;
    i := i + 1
  end ;
  write ( sum )
end .
