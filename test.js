function twoSum(arr, n) {
  for(i = 0; i < arr.length; i++) {
    for (j = i + 1; j < arr.length; j++) {
      if (arr[i] + arr[j] == n) return true;
    }
  }
  return false;
}
let name = 'Mark';

console.log('Hello ' + name + '!');
console.log('The answer is: twoSum([1,3,5], 8));
